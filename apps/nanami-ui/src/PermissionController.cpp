#include "PermissionController.h"

#include "HttpJsonClient.h"
#include "SseStreamParser.h"

#include <QJsonArray>
#include <QJsonObject>
#include <QNetworkReply>
#include <QUrl>

PermissionController::PermissionController(QObject *parent)
    : QObject(parent)
{
    m_lastDecision = tr("none");
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

QString PermissionController::lastDecision() const
{
    return m_lastDecision;
}

QString PermissionController::auditText() const
{
    return m_auditText;
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

    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/permissions/mock/stream")));

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, tr("nanami-core permission stream is unavailable")));
        }
    });
}

void PermissionController::refreshDecision()
{
    if (m_permissionId.isEmpty()) {
        return;
    }

    fetchDecision(m_permissionId);
}

void PermissionController::refreshAuditLog()
{
    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/permissions/audit")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, tr("Failed to fetch permission audit log")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(tr("Invalid permission audit response"));
            return;
        }

        QStringList lines;
        const auto records = object.value(QStringLiteral("records")).toArray();
        for (const QJsonValue &recordValue : records) {
            const auto record = recordValue.toObject();
            QString line = record.value(QStringLiteral("action")).toString()
                + QStringLiteral(" ")
                + record.value(QStringLiteral("permission_id")).toString();

            if (record.contains(QStringLiteral("level"))) {
                line += QStringLiteral(" ") + record.value(QStringLiteral("level")).toString();
            }
            if (record.contains(QStringLiteral("permission_action"))) {
                line += QStringLiteral(" ") + record.value(QStringLiteral("permission_action")).toString();
            }
            if (record.contains(QStringLiteral("target"))) {
                line += tr(" target=") + record.value(QStringLiteral("target")).toString();
            }
            if (record.contains(QStringLiteral("decision"))) {
                line += QStringLiteral(" ") + record.value(QStringLiteral("decision")).toString();
            }
            line += QStringLiteral(" ") + record.value(QStringLiteral("result")).toString();
            lines.append(line);
        }

        m_auditText = lines.join(QStringLiteral("\n"));
        emit auditChanged();
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

void PermissionController::acceptPermissionRequest(const QJsonObject &object)
{
    m_permissionId = object.value(QStringLiteral("permission_id")).toString();
    m_permissionLevel = object.value(QStringLiteral("level")).toString();
    m_permissionAction = object.value(QStringLiteral("action")).toString();
    m_permissionTarget = object.value(QStringLiteral("target")).toString();
    m_permissionReason = object.value(QStringLiteral("reason")).toString();
    m_permissionScope = object.value(QStringLiteral("scope")).toString();
    m_permissionExpires = object.value(QStringLiteral("expires")).toString();
    m_hasPermissionRequest = !m_permissionId.isEmpty();
    m_lastDecision = tr("none");
    emit permissionChanged();
    emit decisionChanged();
    refreshAuditLog();
}

void PermissionController::handleStreamData(const QByteArray &data)
{
    if (data.isEmpty()) {
        return;
    }

    const QStringList payloads = SseStreamParser::extractDataFrames(&m_streamBuffer, data);
    for (const QString &payload : payloads) {
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
}

void PermissionController::resolve(const QString &decision)
{
    if (!m_hasPermissionRequest || m_busy) {
        return;
    }

    setBusy(true);
    setError(QString());

    QJsonObject body;
    const QString resolvedPermissionId = m_permissionId;
    body.insert(QStringLiteral("permission_id"), m_permissionId);
    body.insert(QStringLiteral("decision"), decision);

    HttpJsonClient client(&m_network);
    auto *reply = client.postJson(QUrl(QStringLiteral("http://127.0.0.1:17878/permissions/resolve")), body);

    connect(reply, &QNetworkReply::finished, this, [this, reply, resolvedPermissionId]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, tr("Failed to resolve permission")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(tr("Invalid permission resolve response"));
            return;
        }
        m_lastDecision = object.value(QStringLiteral("decision")).toString(tr("none"));
        emit decisionChanged();
        fetchDecision(resolvedPermissionId);
        refreshAuditLog();
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

void PermissionController::fetchDecision(const QString &permissionId)
{
    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/permissions/decision/") + permissionId));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, tr("Failed to fetch permission decision")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(tr("Invalid permission decision response"));
            return;
        }

        const auto value = object.value(QStringLiteral("decision"));
        m_lastDecision = value.isNull() ? tr("none") : value.toString(tr("none"));
        emit decisionChanged();
    });
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
