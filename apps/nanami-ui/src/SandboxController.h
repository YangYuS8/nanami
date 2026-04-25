#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class SandboxController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString sandboxId READ sandboxId NOTIFY sandboxChanged)
    Q_PROPERTY(QString sandboxStatus READ sandboxStatus NOTIFY sandboxChanged)
    Q_PROPERTY(QString templateId READ templateId NOTIFY sandboxChanged)
    Q_PROPERTY(QString networkPolicy READ networkPolicy NOTIFY sandboxChanged)
    Q_PROPERTY(QString mountText READ mountText NOTIFY sandboxChanged)
    Q_PROPERTY(QString outputText READ outputText NOTIFY sandboxChanged)
    Q_PROPERTY(QString artifactText READ artifactText NOTIFY sandboxChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)

public:
    explicit SandboxController(QObject *parent = nullptr);

    QString sandboxId() const;
    QString sandboxStatus() const;
    QString templateId() const;
    QString networkPolicy() const;
    QString mountText() const;
    QString outputText() const;
    QString artifactText() const;
    QString error() const;
    bool busy() const;

    Q_INVOKABLE void startMockSandboxStream();

signals:
    void sandboxChanged();
    void errorChanged();
    void busyChanged();

private:
    void resetState();
    void handleStreamData(const QByteArray &data);
    void handleEvent(const QJsonObject &event);
    void setError(const QString &error);
    void setBusy(bool busy);

    QNetworkAccessManager m_network;
    QString m_streamBuffer;
    QString m_sandboxId;
    QString m_sandboxStatus;
    QString m_templateId;
    QString m_networkPolicy;
    QString m_mountText;
    QString m_outputText;
    QString m_artifactText;
    QString m_error;
    bool m_busy = false;
};
