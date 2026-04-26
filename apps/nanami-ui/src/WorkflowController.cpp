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

QString WorkflowController::applyPatchStatus() const
{
    return m_applyPatchStatus;
}

QString WorkflowController::applyPatchText() const
{
    return m_applyPatchText;
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

void WorkflowController::requestMockApplyPatch()
{
    if (m_busy || m_state.patch.patchId.isEmpty()) {
        return;
    }

    setError(QString());
    setBusy(true);

    QJsonObject body;
    body.insert(QStringLiteral("patch_id"), m_state.patch.patchId);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/workflow/mock/apply-patch")));
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    auto *reply = m_network.post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("Failed to request mock apply patch"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setError(QStringLiteral("Invalid mock apply patch response"));
            return;
        }

        const auto object = document.object();
        m_applyPatchStatus = object.value(QStringLiteral("status")).toString();
        m_applyPatchText = QStringLiteral("%1 (permission_id=%2)")
                               .arg(object.value(QStringLiteral("message")).toString(),
                                    object.value(QStringLiteral("permission_id")).toString());
        emit workflowChanged();
    });
}

void WorkflowController::resetState()
{
    m_state = WorkflowViewState {};
    m_stepText.clear();
    m_testResultText.clear();
    m_patchText.clear();
    m_applyPatchStatus.clear();
    m_applyPatchText.clear();
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
            event.value(QStringLiteral("command_preview")).toString(),
            event.value(QStringLiteral("duration_ms")).toVariant().toString(),
            event.value(QStringLiteral("passed")).toInt(),
            event.value(QStringLiteral("failed")).toInt(),
        };
        m_state.testResult.failedTestNames.clear();
        const auto failedTests = event.value(QStringLiteral("failed_test_names")).toArray();
        for (const auto &value : failedTests) {
            m_state.testResult.failedTestNames.append(value.toString());
        }
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
        m_state.patch.riskLevel = event.value(QStringLiteral("risk_level")).toString();
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
        QStringList resultLines;
        resultLines.append(m_state.testResult.summary);
        resultLines.append(QStringLiteral("command: %1").arg(m_state.testResult.commandPreview));
        resultLines.append(QStringLiteral("duration_ms: %1").arg(m_state.testResult.durationMs));
        resultLines.append(
            QStringLiteral("passed=%1, failed=%2")
                .arg(m_state.testResult.passed)
                .arg(m_state.testResult.failed));
        for (const auto &failedTest : m_state.testResult.failedTestNames) {
            resultLines.append(QStringLiteral("failed test: %1").arg(failedTest));
        }
        m_testResultText = resultLines.join(QStringLiteral("\n"));
    } else {
        m_testResultText.clear();
    }

    QStringList patchLines;
    if (!m_state.patch.summary.isEmpty()) {
        patchLines.append(m_state.patch.summary);
        patchLines.append(m_state.patch.diffSummary);
        patchLines.append(QStringLiteral("risk: %1").arg(m_state.patch.riskLevel));
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
