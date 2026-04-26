#pragma once

#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class QNetworkReply;
class QUrl;

class HttpJsonClient final : public QObject
{
    Q_OBJECT

public:
    explicit HttpJsonClient(QNetworkAccessManager *network, QObject *parent = nullptr);

    QNetworkReply *get(const QUrl &url) const;
    QNetworkReply *postJson(const QUrl &url, const QJsonObject &body) const;
    QNetworkReply *postEmpty(const QUrl &url) const;

    static bool parseObject(QNetworkReply *reply, QJsonObject *object, QString *error);
    static QString networkErrorString(QNetworkReply *reply, const QString &fallback);

private:
    QNetworkAccessManager *m_network;
};
