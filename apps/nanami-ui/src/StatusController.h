#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>
#include <QTimer>

class StatusController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString coreStatus READ coreStatus NOTIFY coreStatusChanged)
    Q_PROPERTY(QString openClawStatus READ openClawStatus NOTIFY openClawStatusChanged)
    Q_PROPERTY(QString openClawGatewayUrl READ openClawGatewayUrl NOTIFY openClawGatewayUrlChanged)
    Q_PROPERTY(QString openClawMessage READ openClawMessage NOTIFY openClawMessageChanged)

public:
    explicit StatusController(QObject *parent = nullptr);

    QString coreStatus() const;
    QString openClawStatus() const;
    QString openClawGatewayUrl() const;
    QString openClawMessage() const;
    Q_INVOKABLE void refresh();

signals:
    void coreStatusChanged();
    void openClawStatusChanged();
    void openClawGatewayUrlChanged();
    void openClawMessageChanged();

private:
    void refreshCoreStatus();
    void refreshOpenClawStatus();
    void setCoreStatus(const QString &status);
    void setOpenClawStatus(const QString &status);
    void setOpenClawGatewayUrl(const QString &gatewayUrl);
    void setOpenClawMessage(const QString &message);

    QNetworkAccessManager m_network;
    QTimer m_refreshTimer;
    QString m_coreStatus = QStringLiteral("disconnected");
    QString m_openClawStatus = QStringLiteral("disconnected");
    QString m_openClawGatewayUrl;
    QString m_openClawMessage;
};
