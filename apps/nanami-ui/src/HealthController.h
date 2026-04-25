#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>
#include <QTimer>

class HealthController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString status READ status NOTIFY statusChanged)

public:
    explicit HealthController(QObject *parent = nullptr);

    QString status() const;
    Q_INVOKABLE void refresh();

signals:
    void statusChanged();

private:
    void setStatus(const QString &status);

    QNetworkAccessManager m_network;
    QTimer m_refreshTimer;
    QString m_status = QStringLiteral("disconnected");
};
