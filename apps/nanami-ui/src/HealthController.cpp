#include "HealthController.h"

#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

HealthController::HealthController(QObject *parent)
    : QObject(parent)
{
    m_refreshTimer.setInterval(3000);
    connect(&m_refreshTimer, &QTimer::timeout, this, &HealthController::refresh);
    m_refreshTimer.start();
}

QString HealthController::status() const
{
    return m_status;
}

void HealthController::refresh()
{
    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/health")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();

        if (reply->error() != QNetworkReply::NoError) {
            setStatus(QStringLiteral("disconnected"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setStatus(QStringLiteral("error"));
            return;
        }

        const auto object = document.object();
        setStatus(object.value(QStringLiteral("status")).toString() == QStringLiteral("ok")
                      ? QStringLiteral("connected")
                      : QStringLiteral("error"));
    });
}

void HealthController::setStatus(const QString &status)
{
    if (m_status == status) {
        return;
    }

    m_status = status;
    emit statusChanged();
}
