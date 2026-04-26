#include "SandboxController.h"

#include "HttpJsonClient.h"
#include "SseStreamParser.h"

#include <QJsonArray>
#include <QJsonObject>
#include <QNetworkReply>
#include <QUrl>

SandboxController::SandboxController(QObject *parent)
    : QObject(parent)
{
}

QString SandboxController::sandboxId() const
{
    return m_state.sandboxId;
}

QString SandboxController::sandboxStatus() const
{
    return m_state.status;
}

QString SandboxController::templateId() const
{
    return m_state.templateId;
}

QString SandboxController::networkPolicy() const
{
    return m_state.networkPolicy;
}

QString SandboxController::mountText() const
{
    return m_mountText;
}

QString SandboxController::outputText() const
{
    return m_outputText;
}

QString SandboxController::artifactText() const
{
    return m_artifactText;
}

QString SandboxController::error() const
{
    return m_error;
}

bool SandboxController::busy() const
{
    return m_busy;
}

void SandboxController::startMockSandboxStream()
{
    if (m_busy) {
        return;
    }

    resetState();
    m_streamBuffer.clear();
    setError(QString());
    setBusy(true);

    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/sandbox/mock/stream")));

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("nanami-core mock sandbox stream is unavailable")));
        }
    });
}

void SandboxController::resetState()
{
    m_state = SandboxViewState {};
    m_mountText.clear();
    m_outputText.clear();
    m_artifactText.clear();
    emit sandboxChanged();
}

void SandboxController::handleStreamData(const QByteArray &data)
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

void SandboxController::handleEvent(const QJsonObject &event)
{
    const QString type = event.value(QStringLiteral("type")).toString();

    if (type == QStringLiteral("sandbox.started")) {
        handleSandboxStarted(event);
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.updated")) {
        handleSandboxUpdated(event);
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.output")) {
        handleSandboxOutput(event);
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.artifact")) {
        handleSandboxArtifact(event);
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.completed")) {
        handleSandboxCompleted(event);
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("error.occurred")) {
        setError(event.value(QStringLiteral("message")).toString(QStringLiteral("Mock sandbox stream failed")));
    }
}

void SandboxController::handleSandboxStarted(const QJsonObject &event)
{
    m_state.sandboxId = event.value(QStringLiteral("sandbox_id")).toString();
    m_state.taskId = event.value(QStringLiteral("task_id")).toString();
    m_state.templateId = event.value(QStringLiteral("template_id")).toString();
    m_state.status = event.value(QStringLiteral("status")).toString();
    m_state.networkPolicy = event.value(QStringLiteral("network_policy")).toString();
    m_state.mounts.clear();

    const auto mountArray = event.value(QStringLiteral("mounts")).toArray();
    for (const auto &mountValue : mountArray) {
        const auto mount = mountValue.toObject();
        m_state.mounts.append(SandboxMountView {
            mount.value(QStringLiteral("host_path")).toString(),
            mount.value(QStringLiteral("sandbox_path")).toString(),
            mount.value(QStringLiteral("mode")).toString(),
        });
    }

    rebuildDerivedText();
}

void SandboxController::handleSandboxUpdated(const QJsonObject &event)
{
    m_state.sandboxId = event.value(QStringLiteral("sandbox_id")).toString(m_state.sandboxId);
    m_state.taskId = event.value(QStringLiteral("task_id")).toString(m_state.taskId);
    m_state.status = event.value(QStringLiteral("status")).toString(m_state.status);
    const QString summary = event.value(QStringLiteral("summary")).toString();
    if (!summary.isEmpty()) {
        m_state.summary = summary;
    }

    rebuildDerivedText();
}

void SandboxController::handleSandboxOutput(const QJsonObject &event)
{
    m_state.sandboxId = event.value(QStringLiteral("sandbox_id")).toString(m_state.sandboxId);
    m_state.taskId = event.value(QStringLiteral("task_id")).toString(m_state.taskId);
    m_state.outputs.append(SandboxOutputView {
        event.value(QStringLiteral("stream")).toString(),
        event.value(QStringLiteral("content")).toString(),
    });

    rebuildDerivedText();
}

void SandboxController::handleSandboxArtifact(const QJsonObject &event)
{
    m_state.sandboxId = event.value(QStringLiteral("sandbox_id")).toString(m_state.sandboxId);
    m_state.taskId = event.value(QStringLiteral("task_id")).toString(m_state.taskId);
    m_state.artifacts.append(SandboxArtifactView {
        event.value(QStringLiteral("name")).toString(),
        event.value(QStringLiteral("path")).toString(),
        event.value(QStringLiteral("media_type")).toString(),
        event.value(QStringLiteral("size_bytes")).toVariant().toString(),
    });

    rebuildDerivedText();
}

void SandboxController::handleSandboxCompleted(const QJsonObject &event)
{
    m_state.sandboxId = event.value(QStringLiteral("sandbox_id")).toString(m_state.sandboxId);
    m_state.taskId = event.value(QStringLiteral("task_id")).toString(m_state.taskId);
    m_state.status = event.value(QStringLiteral("status")).toString(m_state.status);
    if (event.contains(QStringLiteral("exit_code"))) {
        m_state.exitCode = event.value(QStringLiteral("exit_code")).toVariant().toString();
    }
    const QString summary = event.value(QStringLiteral("summary")).toString();
    if (!summary.isEmpty()) {
        m_state.summary = summary;
    }

    rebuildDerivedText();
}

void SandboxController::rebuildDerivedText()
{
    QStringList mounts;
    for (const auto &mount : m_state.mounts) {
        mounts.append(QStringLiteral("%1 -> %2 (%3)")
                          .arg(mount.hostPath, mount.sandboxPath, mount.mode));
    }
    m_mountText = mounts.join(QStringLiteral("\n"));

    QStringList outputs;
    for (const auto &output : m_state.outputs) {
        outputs.append(QStringLiteral("%1: %2").arg(output.stream, output.content));
    }
    if (!m_state.summary.isEmpty()) {
        outputs.append(QStringLiteral("summary: %1").arg(m_state.summary));
    }
    m_outputText = outputs.join(QStringLiteral("\n"));

    QStringList artifacts;
    for (const auto &artifact : m_state.artifacts) {
        artifacts.append(QStringLiteral("%1 (%2, %3 bytes) @ %4")
                             .arg(artifact.name, artifact.mediaType, artifact.sizeBytes, artifact.path));
    }
    m_artifactText = artifacts.join(QStringLiteral("\n"));
}

void SandboxController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}

void SandboxController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}
