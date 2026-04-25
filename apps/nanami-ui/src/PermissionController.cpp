#include "PermissionController.h"

#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

PermissionController::PermissionController(QObject *parent)
    : QObject(parent)
{
}

bool PermissionController::hasPermissionRequest() const
{
    return m_hasPermissionRequest;
}

QString PermissionController::permissionId() const
{
    return m_permissionId;
}

QString PermissionController::permissionLevel() const
{
    return m_permissionLevel;
}

QString PermissionController::permissionAction() const
{
    return m_permissionAction;
}

QString PermissionController::permissionTarget() const
{
    return m_permissionTarget;
}

QString PermissionController::permissionReason() const
{
    return m_permissionReason;
}

QString PermissionController::permissionScope() const
{
    return m_permissionScope;
}

QString PermissionController::permissionExpires() const
{
    return m_permissionExpires;
}

QString PermissionController::error() const
{
    return m_error;
}

bool PermissionController::busy() const
{
    return m_busy;
}

void PermissionController::startMockPermissionStream()
{
    if (m_busy) {
        return;
    }

    m_streamBuffer.clear();
    clearRequest();
    setError(QString());
    setBusy(true);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/permissions/mock/stream")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("nanami-core permission stream is unavailable"));
        }
    });
}

void PermissionController::resolveAllowOnce()
{
    resolve(QStringLiteral("allow_once"));
}

void PermissionController::resolveAllowForTask()
{
    resolve(QStringLiteral("allow_for_task"));
}

void PermissionController::resolveDeny()
{
    resolve(QStringLiteral("deny"));
}

void PermissionController::handleStreamData(const QByteArray &data)
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
                const QJsonObject object = document.object();
                if (object.value(QStringLiteral("type")).toString() == QStringLiteral("permission.requested")) {
                    m_permissionId = object.value(QStringLiteral("permission_id")).toString();
                    m_permissionLevel = object.value(QStringLiteral("level")).toString();
                    m_permissionAction = object.value(QStringLiteral("action")).toString();
                    m_permissionTarget = object.value(QStringLiteral("target")).toString();
                    m_permissionReason = object.value(QStringLiteral("reason")).toString();
                    m_permissionScope = object.value(QStringLiteral("scope")).toString();
                    m_permissionExpires = object.value(QStringLiteral("expires")).toString();
                    m_hasPermissionRequest = true;
                    emit permissionChanged();
                }
            }
        }

        separator = m_streamBuffer.indexOf(QStringLiteral("\n\n"));
    }
}

void PermissionController::resolve(const QString &decision)
{
    if (!m_hasPermissionRequest || m_busy) {
        return;
    }

    setBusy(true);
    setError(QString());

    QJsonObject body;
    body.insert(QStringLiteral("permission_id"), m_permissionId);
    body.insert(QStringLiteral("decision"), decision);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/permissions/resolve")));
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    auto *reply = m_network.post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("Failed to resolve permission"));
            return;
        }

        if (!document.isObject()) {
            setError(QStringLiteral("Invalid permission resolve response"));
            return;
        }

        clearRequest();
    });
}

void PermissionController::clearRequest()
{
    m_hasPermissionRequest = false;
    m_permissionId.clear();
    m_permissionLevel.clear();
    m_permissionAction.clear();
    m_permissionTarget.clear();
    m_permissionReason.clear();
    m_permissionScope.clear();
    m_permissionExpires.clear();
    emit permissionChanged();
}

void PermissionController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}

void PermissionController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}
