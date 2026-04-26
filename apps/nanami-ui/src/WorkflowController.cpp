#include "WorkflowController.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

WorkflowController::WorkflowController(QObject *parent)
    : QObject(parent)
{
}

QString WorkflowController::workflowId() const
{
    return m_state.workflowId;
}

QString WorkflowController::workflowStatus() const
{
    return m_state.workflowStatus;
}

QString WorkflowController::projectPath() const
{
    return m_state.projectPath;
}

QString WorkflowController::stepText() const
{
    return m_stepText;
}

QString WorkflowController::testResultText() const
{
    return m_testResultText;
}

QString WorkflowController::patchText() const
{
    return m_patchText;
}

bool WorkflowController::busy() const
{
    return m_busy;
}

QString WorkflowController::error() const
{
    return m_error;
}

void WorkflowController::startMockWorkflowStream()
{
    if (m_busy) {
        return;
    }

    resetState();
    m_streamBuffer.clear();
    setError(QString());
    setBusy(true);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/workflow/mock/stream")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("nanami-core mock workflow stream is unavailable"));
        }
    });
}

void WorkflowController::resetState()
{
    m_state = WorkflowViewState {};
    m_stepText.clear();
    m_testResultText.clear();
    m_patchText.clear();
    emit workflowChanged();
}

void WorkflowController::handleStreamData(const QByteArray &data)
{
    if (data.isEmpty()) {
        return;
    }

    m_streamBuffer.append(QString::fromUtf8(data));
    int separator = m_streamBuffer.indexOf(QStringLiteral("\n\n"));
    while (separator >= 0) {
        const QString frame = m_streamBuffer.left(separator).trimmed();
        m_streamBuffer.remove(0, separator + 2);

        if (frame.startsWith(QStringLiteral("data:"))) {
            const QString payload = frame.mid(5).trimmed();
            const auto document = QJsonDocument::fromJson(payload.toUtf8());
            if (document.isObject()) {
                handleEvent(document.object());
            }
        }

        separator = m_streamBuffer.indexOf(QStringLiteral("\n\n"));
    }
}

void WorkflowController::handleEvent(const QJsonObject &event)
{
    const QString type = event.value(QStringLiteral("type")).toString();

    if (type == QStringLiteral("workflow.started")) {
        m_state.workflowId = event.value(QStringLiteral("workflow_id")).toString();
        m_state.workflowStatus = event.value(QStringLiteral("status")).toString();
        m_state.projectPath = event.value(QStringLiteral("project_path")).toString();
        rebuildDerivedText();
        emit workflowChanged();
        return;
    }

    if (type == QStringLiteral("workflow.step")) {
        m_state.workflowId = event.value(QStringLiteral("workflow_id")).toString(m_state.workflowId);
        m_state.steps.append(WorkflowStepView {
            event.value(QStringLiteral("step_kind")).toString(),
            event.value(QStringLiteral("status")).toString(),
            event.value(QStringLiteral("summary")).toString(),
        });
        if (event.contains(QStringLiteral("status"))) {
            m_state.workflowStatus = event.value(QStringLiteral("status")).toString(m_state.workflowStatus);
        }
        rebuildDerivedText();
        emit workflowChanged();
        return;
    }

    if (type == QStringLiteral("workflow.test_result")) {
        m_state.workflowId = event.value(QStringLiteral("workflow_id")).toString(m_state.workflowId);
        m_state.testResult = WorkflowTestResultView {
            event.value(QStringLiteral("status")).toString(),
            event.value(QStringLiteral("summary")).toString(),
            event.value(QStringLiteral("passed")).toInt(),
            event.value(QStringLiteral("failed")).toInt(),
        };
        m_state.workflowStatus = event.value(QStringLiteral("status")).toString(m_state.workflowStatus);
        rebuildDerivedText();
        emit workflowChanged();
        return;
    }

    if (type == QStringLiteral("workflow.patch_proposed")) {
        m_state.workflowId = event.value(QStringLiteral("workflow_id")).toString(m_state.workflowId);
        m_state.patch.patchId = event.value(QStringLiteral("patch_id")).toString();
        m_state.patch.summary = event.value(QStringLiteral("summary")).toString();
        m_state.patch.diffSummary = event.value(QStringLiteral("diff_summary")).toString();
        m_state.patch.files.clear();

        const auto files = event.value(QStringLiteral("files")).toArray();
        for (const auto &value : files) {
            const auto file = value.toObject();
            m_state.patch.files.append(WorkflowPatchFileView {
                file.value(QStringLiteral("path")).toString(),
                file.value(QStringLiteral("change_type")).toString(),
                file.value(QStringLiteral("diff_preview")).toString(),
            });
        }

        rebuildDerivedText();
        emit workflowChanged();
        return;
    }

    if (type == QStringLiteral("workflow.completed")) {
        m_state.workflowId = event.value(QStringLiteral("workflow_id")).toString(m_state.workflowId);
        m_state.workflowStatus = event.value(QStringLiteral("status")).toString(m_state.workflowStatus);
        m_state.steps.append(WorkflowStepView {
            QStringLiteral("completed"),
            event.value(QStringLiteral("status")).toString(),
            event.value(QStringLiteral("summary")).toString(),
        });
        rebuildDerivedText();
        emit workflowChanged();
        return;
    }

    if (type == QStringLiteral("error.occurred")) {
        setError(event.value(QStringLiteral("message")).toString(QStringLiteral("Mock workflow stream failed")));
    }
}

void WorkflowController::rebuildDerivedText()
{
    QStringList stepLines;
    for (const auto &step : m_state.steps) {
        stepLines.append(QStringLiteral("%1: %2 (%3)").arg(step.kind, step.summary, step.status));
    }
    m_stepText = stepLines.join(QStringLiteral("\n"));

    if (!m_state.testResult.summary.isEmpty()) {
        m_testResultText = QStringLiteral("%1 (passed=%2, failed=%3)")
                               .arg(m_state.testResult.summary)
                               .arg(m_state.testResult.passed)
                               .arg(m_state.testResult.failed);
    } else {
        m_testResultText.clear();
    }

    QStringList patchLines;
    if (!m_state.patch.summary.isEmpty()) {
        patchLines.append(m_state.patch.summary);
        patchLines.append(m_state.patch.diffSummary);
        for (const auto &file : m_state.patch.files) {
            patchLines.append(QStringLiteral("%1 [%2]").arg(file.path, file.changeType));
            patchLines.append(file.diffPreview);
        }
    }
    m_patchText = patchLines.join(QStringLiteral("\n"));
}

void WorkflowController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}

void WorkflowController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}
