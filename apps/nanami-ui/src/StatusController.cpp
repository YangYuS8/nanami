#include "StatusController.h"

#include "HttpJsonClient.h"

#include <QJsonObject>
#include <QNetworkReply>
#include <QUrl>

StatusController::StatusController(QObject *parent)
    : QObject(parent)
{
    m_refreshTimer.setInterval(3000);
    connect(&m_refreshTimer, &QTimer::timeout, this, &StatusController::refresh);
    m_refreshTimer.start();
}

QString StatusController::coreStatus() const
{
    return m_coreStatus;
}

QString StatusController::openClawStatus() const
{
    return m_openClawStatus;
}

QString StatusController::openClawGatewayUrl() const
{
    return m_openClawGatewayUrl;
}

QString StatusController::openClawMessage() const
{
    return m_openClawMessage;
}

void StatusController::refresh()
{
    refreshCoreStatus();
    refreshOpenClawStatus();
}

void StatusController::refreshCoreStatus()
{
    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/health")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();

        if (reply->error() != QNetworkReply::NoError) {
            setCoreStatus(QStringLiteral("disconnected"));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setCoreStatus(QStringLiteral("error"));
            return;
        }

        setCoreStatus(object.value(QStringLiteral("status")).toString() == QStringLiteral("ok")
                          ? QStringLiteral("connected")
                          : QStringLiteral("error"));
    });
}

void StatusController::refreshOpenClawStatus()
{
    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/openclaw/status")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();

        if (reply->error() != QNetworkReply::NoError) {
            setOpenClawStatus(QStringLiteral("disconnected"));
            setOpenClawMessage(QStringLiteral("nanami-core is unavailable"));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setOpenClawStatus(QStringLiteral("error"));
            setOpenClawMessage(QStringLiteral("Invalid nanami-core OpenClaw status response"));
            return;
        }

        setOpenClawStatus(object.value(QStringLiteral("status")).toString(QStringLiteral("error")));
        setOpenClawGatewayUrl(object.value(QStringLiteral("gateway_url")).toString());
        setOpenClawMessage(object.value(QStringLiteral("message")).toString());
    });
}

void StatusController::setCoreStatus(const QString &status)
{
    if (m_coreStatus == status) {
        return;
    }

    m_coreStatus = status;
    emit coreStatusChanged();
}

void StatusController::setOpenClawStatus(const QString &status)
{
    if (m_openClawStatus == status) {
        return;
    }

    m_openClawStatus = status;
    emit openClawStatusChanged();
}

void StatusController::setOpenClawGatewayUrl(const QString &gatewayUrl)
{
    if (m_openClawGatewayUrl == gatewayUrl) {
        return;
    }

    m_openClawGatewayUrl = gatewayUrl;
    emit openClawGatewayUrlChanged();
}

void StatusController::setOpenClawMessage(const QString &message)
{
    if (m_openClawMessage == message) {
        return;
    }

    m_openClawMessage = message;
    emit openClawMessageChanged();
}
