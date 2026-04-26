#include "TaskController.h"

#include "HttpJsonClient.h"
#include "SseStreamParser.h"

#include <QJsonObject>
#include <QNetworkReply>
#include <QUrl>

TaskController::TaskController(QObject *parent)
    : QObject(parent)
{
}

QString TaskController::taskTimelineText() const
{
    return m_taskTimelineText;
}

QString TaskController::currentTaskId() const
{
    return m_currentTask.taskId;
}

QString TaskController::currentTaskStatus() const
{
    return m_currentTask.status;
}

QString TaskController::currentTaskTitle() const
{
    return m_currentTask.title;
}

int TaskController::toolCount() const
{
    return m_currentTask.tools.size();
}

QString TaskController::error() const
{
    return m_error;
}

bool TaskController::busy() const
{
    return m_busy;
}

void TaskController::startMockTaskStream()
{
    if (m_busy) {
        return;
    }

    resetState();
    m_streamBuffer.clear();
    setError(QString());
    setBusy(true);

    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/tasks/mock/stream")));

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, tr("nanami-core mock task stream is unavailable")));
        }
    });
}

void TaskController::startOpenClawTaskStream(const QString &message)
{
    const QString trimmed = message.trimmed();
    if (trimmed.isEmpty() || m_busy) {
        return;
    }

    resetState();
    m_streamBuffer.clear();
    setError(QString());
    setBusy(true);

    QJsonObject body;
    body.insert(QStringLiteral("message"), trimmed);

    HttpJsonClient client(&m_network);
    auto *reply = client.postJson(
        QUrl(QStringLiteral("http://127.0.0.1:17878/tasks/openclaw/stream")), body);

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, tr("nanami-core OpenClaw task stream is unavailable")));
        }
    });
}

void TaskController::resetState()
{
    m_currentTask = TaskViewState {};
    m_permissionLines.clear();
    m_activityLines.clear();
    m_taskTimelineText.clear();
    emit currentTaskChanged();
    emit taskTimelineTextChanged();
}

void TaskController::rebuildTimeline()
{
    QStringList lines;

    if (!m_currentTask.taskId.isEmpty()) {
        lines.append(tr("Task %1 started: %2")
                         .arg(m_currentTask.taskId, m_currentTask.title));
    }

    for (const QString &toolId : m_currentTask.toolOrder) {
        const ToolViewState tool = m_currentTask.tools.value(toolId);
        lines.append(tr("Tool %1 started: %2")
                         .arg(tool.toolCallId, tool.tool));

        for (const ToolOutputView &output : tool.outputs) {
            lines.append(QStringLiteral("%1: %2").arg(output.stream, output.content));
        }

        if (!tool.status.isEmpty()) {
            QString completion = tr("Tool %1 completed: status=%2")
                                     .arg(tool.toolCallId, tool.status);
            if (!tool.exitCode.isEmpty()) {
                completion.append(tr(", exit_code=%1").arg(tool.exitCode));
            }
            lines.append(completion);
        }
    }

    if (!m_currentTask.taskId.isEmpty()
        && (m_currentTask.status == QStringLiteral("completed")
            || m_currentTask.status == QStringLiteral("failed")
            || m_currentTask.status == QStringLiteral("cancelled"))) {
        lines.append(tr("Task %1 completed: %2")
                         .arg(m_currentTask.taskId,
                              m_currentTask.summary.isEmpty() ? m_currentTask.status : m_currentTask.summary));
    }

    lines.append(m_permissionLines);
    lines.append(m_activityLines);

    m_taskTimelineText = lines.join(QStringLiteral("\n"));
    emit taskTimelineTextChanged();
}

void TaskController::handleStreamData(const QByteArray &data)
{
    if (data.isEmpty()) {
        return;
    }

    const QStringList payloads = SseStreamParser::extractDataFrames(&m_streamBuffer, data);
    for (const QString &payload : payloads) {
        const auto document = QJsonDocument::fromJson(payload.toUtf8());
        if (document.isObject()) {
            handleEvent(document.object());
        }
    }
}

void TaskController::handleEvent(const QJsonObject &event)
{
    const QString type = event.value(QStringLiteral("type")).toString();

    if (type == QStringLiteral("task.started")) {
        m_currentTask.taskId = event.value(QStringLiteral("task_id")).toString();
        m_currentTask.title = event.value(QStringLiteral("title")).toString();
        m_currentTask.status = event.value(QStringLiteral("status")).toString();
        rebuildTimeline();
        emit currentTaskChanged();
        return;
    }

    if (type == QStringLiteral("task.updated")) {
        m_currentTask.status = event.value(QStringLiteral("status")).toString();
        m_currentTask.summary = event.value(QStringLiteral("summary")).toString();
        rebuildTimeline();
        emit currentTaskChanged();
        return;
    }

    if (type == QStringLiteral("tool.started")) {
        ToolViewState tool;
        const QString toolCallId = event.value(QStringLiteral("tool_call_id")).toString();
        tool.toolCallId = toolCallId;
        tool.tool = event.value(QStringLiteral("tool")).toString();
        m_currentTask.tools.insert(toolCallId, tool);
        if (!m_currentTask.toolOrder.contains(toolCallId)) {
            m_currentTask.toolOrder.append(toolCallId);
        }
        rebuildTimeline();
        emit currentTaskChanged();
        return;
    }

    if (type == QStringLiteral("tool.output")) {
        const QString toolCallId = event.value(QStringLiteral("tool_call_id")).toString();
        ToolViewState tool = m_currentTask.tools.value(toolCallId);
        tool.outputs.append(ToolOutputView {
            event.value(QStringLiteral("stream")).toString(),
            event.value(QStringLiteral("content")).toString(),
        });
        m_currentTask.tools.insert(toolCallId, tool);
        if (!m_currentTask.toolOrder.contains(toolCallId)) {
            m_currentTask.toolOrder.append(toolCallId);
        }
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("tool.completed")) {
        const QString toolCallId = event.value(QStringLiteral("tool_call_id")).toString();
        ToolViewState tool = m_currentTask.tools.value(toolCallId);
        tool.status = event.value(QStringLiteral("status")).toString();
        tool.exitCode = event.value(QStringLiteral("exit_code")).toVariant().toString();
        m_currentTask.tools.insert(toolCallId, tool);
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("task.completed")) {
        m_currentTask.status = event.value(QStringLiteral("status")).toString();
        m_currentTask.summary = event.value(QStringLiteral("summary")).toString();
        rebuildTimeline();
        emit currentTaskChanged();
        return;
    }

    if (type == QStringLiteral("permission.requested")) {
        m_permissionLines.append(tr("Permission requested: %1 %2 target=%3")
                                     .arg(event.value(QStringLiteral("level")).toString(),
                                          event.value(QStringLiteral("action")).toString(),
                                          event.value(QStringLiteral("target")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("sandbox.started")) {
        m_activityLines.append(tr("Sandbox %1 started: template=%2 network=%3")
                                   .arg(event.value(QStringLiteral("sandbox_id")).toString(),
                                        event.value(QStringLiteral("template_id")).toString(),
                                        event.value(QStringLiteral("network_policy")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("sandbox.updated")) {
        QString line = tr("Sandbox %1 updated: status=%2")
                           .arg(event.value(QStringLiteral("sandbox_id")).toString(),
                                event.value(QStringLiteral("status")).toString());
        const QString summary = event.value(QStringLiteral("summary")).toString();
        if (!summary.isEmpty()) {
            line.append(tr(", summary=%1").arg(summary));
        }
        m_activityLines.append(line);
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("sandbox.output")) {
        m_activityLines.append(tr("Sandbox %1 %2: %3")
                                   .arg(event.value(QStringLiteral("sandbox_id")).toString(),
                                        event.value(QStringLiteral("stream")).toString(),
                                        event.value(QStringLiteral("content")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("sandbox.artifact")) {
        m_activityLines.append(tr("Sandbox %1 artifact: %2 @ %3")
                                   .arg(event.value(QStringLiteral("sandbox_id")).toString(),
                                        event.value(QStringLiteral("name")).toString(),
                                        event.value(QStringLiteral("path")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("sandbox.completed")) {
        QString line = tr("Sandbox %1 completed: status=%2")
                           .arg(event.value(QStringLiteral("sandbox_id")).toString(),
                                event.value(QStringLiteral("status")).toString());
        if (event.contains(QStringLiteral("exit_code"))) {
            line.append(
                tr(", exit_code=%1").arg(event.value(QStringLiteral("exit_code")).toVariant().toString()));
        }
        const QString summary = event.value(QStringLiteral("summary")).toString();
        if (!summary.isEmpty()) {
            line.append(tr(", summary=%1").arg(summary));
        }
        m_activityLines.append(line);
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("workflow.started")) {
        m_activityLines.append(
            tr("Workflow started: workflow_id=%1, project=%2, status=%3")
                .arg(event.value(QStringLiteral("workflow_id")).toString(),
                     event.value(QStringLiteral("project_path")).toString(),
                     event.value(QStringLiteral("status")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("workflow.step")) {
        m_activityLines.append(
            tr("Workflow step: %1 %2 (%3)")
                .arg(event.value(QStringLiteral("step_kind")).toString(),
                     event.value(QStringLiteral("summary")).toString(),
                     event.value(QStringLiteral("status")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("workflow.test_result")) {
        m_activityLines.append(
            tr("Workflow test result: %1, passed=%2, failed=%3, command=%4")
                .arg(event.value(QStringLiteral("summary")).toString(),
                     event.value(QStringLiteral("passed")).toVariant().toString(),
                     event.value(QStringLiteral("failed")).toVariant().toString(),
                     event.value(QStringLiteral("command_preview")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("workflow.patch_proposed")) {
        m_activityLines.append(
            tr("Workflow patch proposed: patch_id=%1, %2, risk=%3")
                .arg(event.value(QStringLiteral("patch_id")).toString(),
                     event.value(QStringLiteral("diff_summary")).toString(),
                     event.value(QStringLiteral("risk_level")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("workflow.completed")) {
        m_activityLines.append(
            tr("Workflow completed: %1 (%2)")
                .arg(event.value(QStringLiteral("summary")).toString(),
                     event.value(QStringLiteral("status")).toString()));
        rebuildTimeline();
        return;
    }

    if (type == QStringLiteral("error.occurred")) {
        setError(event.value(QStringLiteral("message")).toString(tr("OpenClaw task stream failed")));
    }
}

void TaskController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}

void TaskController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}
