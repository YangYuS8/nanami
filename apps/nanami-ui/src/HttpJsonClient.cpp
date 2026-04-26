#include "HttpJsonClient.h"

#include <QCoreApplication>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

HttpJsonClient::HttpJsonClient(QNetworkAccessManager *network, QObject *parent)
    : QObject(parent)
    , m_network(network)
{
}

QNetworkReply *HttpJsonClient::get(const QUrl &url) const
{
    QNetworkRequest request(url);
    return m_network->get(request);
}

QNetworkReply *HttpJsonClient::postJson(const QUrl &url, const QJsonObject &body) const
{
    QNetworkRequest request(url);
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    return m_network->post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));
}

QNetworkReply *HttpJsonClient::postEmpty(const QUrl &url) const
{
    QNetworkRequest request(url);
    return m_network->post(request, QByteArray());
}

bool HttpJsonClient::parseObject(QNetworkReply *reply, QJsonObject *object, QString *error)
{
    const auto document = QJsonDocument::fromJson(reply->readAll());
    if (!document.isObject()) {
        if (error != nullptr) {
            *error = QCoreApplication::translate("HttpJsonClient", "Invalid JSON object response");
        }
        return false;
    }

    *object = document.object();
    return true;
}

QString HttpJsonClient::networkErrorString(QNetworkReply *reply, const QString &fallback)
{
    if (reply->error() == QNetworkReply::NoError) {
        return fallback;
    }

    const QString detail = reply->errorString().trimmed();
    if (detail.isEmpty()) {
        return fallback;
    }

    return fallback + QStringLiteral(": ") + detail;
}
