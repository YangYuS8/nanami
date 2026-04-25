#include "SandboxController.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

SandboxController::SandboxController(QObject *parent)
    : QObject(parent)
{
}

QString SandboxController::sandboxId() const
{
    return m_sandboxId;
}

QString SandboxController::sandboxStatus() const
{
    return m_sandboxStatus;
}

QString SandboxController::templateId() const
{
    return m_templateId;
}

QString SandboxController::networkPolicy() const
{
    return m_networkPolicy;
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

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/sandbox/mock/stream")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("nanami-core mock sandbox stream is unavailable"));
        }
    });
}

void SandboxController::resetState()
{
    m_sandboxId.clear();
    m_sandboxStatus.clear();
    m_templateId.clear();
    m_networkPolicy.clear();
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

void SandboxController::handleEvent(const QJsonObject &event)
{
    const QString type = event.value(QStringLiteral("type")).toString();

    if (type == QStringLiteral("sandbox.started")) {
        m_sandboxId = event.value(QStringLiteral("sandbox_id")).toString();
        m_sandboxStatus = event.value(QStringLiteral("status")).toString();
        m_templateId = event.value(QStringLiteral("template_id")).toString();
        m_networkPolicy = event.value(QStringLiteral("network_policy")).toString();

        QStringList mounts;
        const auto mountArray = event.value(QStringLiteral("mounts")).toArray();
        for (const auto &mountValue : mountArray) {
            const auto mount = mountValue.toObject();
            mounts.append(QStringLiteral("%1 -> %2 (%3)")
                              .arg(mount.value(QStringLiteral("host_path")).toString(),
                                   mount.value(QStringLiteral("sandbox_path")).toString(),
                                   mount.value(QStringLiteral("mode")).toString()));
        }
        m_mountText = mounts.join(QStringLiteral("\n"));
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.updated")) {
        m_sandboxId = event.value(QStringLiteral("sandbox_id")).toString(m_sandboxId);
        m_sandboxStatus = event.value(QStringLiteral("status")).toString();
        const QString summary = event.value(QStringLiteral("summary")).toString();
        if (!summary.isEmpty()) {
            if (!m_outputText.isEmpty()) {
                m_outputText.append(QStringLiteral("\n"));
            }
            m_outputText.append(QStringLiteral("log: %1").arg(summary));
        }
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.output")) {
        const QString line = QStringLiteral("%1: %2")
                                 .arg(event.value(QStringLiteral("stream")).toString(),
                                      event.value(QStringLiteral("content")).toString());
        if (!m_outputText.isEmpty()) {
            m_outputText.append(QStringLiteral("\n"));
        }
        m_outputText.append(line);
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.artifact")) {
        const QString line = QStringLiteral("%1 (%2, %3 bytes) @ %4")
                                 .arg(event.value(QStringLiteral("name")).toString(),
                                      event.value(QStringLiteral("media_type")).toString(),
                                      event.value(QStringLiteral("size_bytes")).toVariant().toString(),
                                      event.value(QStringLiteral("path")).toString());
        if (!m_artifactText.isEmpty()) {
            m_artifactText.append(QStringLiteral("\n"));
        }
        m_artifactText.append(line);
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("sandbox.completed")) {
        m_sandboxId = event.value(QStringLiteral("sandbox_id")).toString(m_sandboxId);
        m_sandboxStatus = event.value(QStringLiteral("status")).toString();
        const QString summary = event.value(QStringLiteral("summary")).toString();
        if (!summary.isEmpty()) {
            if (!m_outputText.isEmpty()) {
                m_outputText.append(QStringLiteral("\n"));
            }
            m_outputText.append(QStringLiteral("summary: %1").arg(summary));
        }
        emit sandboxChanged();
        return;
    }

    if (type == QStringLiteral("error.occurred")) {
        setError(event.value(QStringLiteral("message")).toString(QStringLiteral("Mock sandbox stream failed")));
    }
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
