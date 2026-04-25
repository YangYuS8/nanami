#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class PermissionController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool hasPermissionRequest READ hasPermissionRequest NOTIFY permissionChanged)
    Q_PROPERTY(QString permissionId READ permissionId NOTIFY permissionChanged)
    Q_PROPERTY(QString permissionLevel READ permissionLevel NOTIFY permissionChanged)
    Q_PROPERTY(QString permissionAction READ permissionAction NOTIFY permissionChanged)
    Q_PROPERTY(QString permissionTarget READ permissionTarget NOTIFY permissionChanged)
    Q_PROPERTY(QString permissionReason READ permissionReason NOTIFY permissionChanged)
    Q_PROPERTY(QString permissionScope READ permissionScope NOTIFY permissionChanged)
    Q_PROPERTY(QString permissionExpires READ permissionExpires NOTIFY permissionChanged)
    Q_PROPERTY(QString lastDecision READ lastDecision NOTIFY decisionChanged)
    Q_PROPERTY(QString auditText READ auditText NOTIFY auditChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)

public:
    explicit PermissionController(QObject *parent = nullptr);

    bool hasPermissionRequest() const;
    QString permissionId() const;
    QString permissionLevel() const;
    QString permissionAction() const;
    QString permissionTarget() const;
    QString permissionReason() const;
    QString permissionScope() const;
    QString permissionExpires() const;
    QString lastDecision() const;
    QString auditText() const;
    QString error() const;
    bool busy() const;

    Q_INVOKABLE void startMockPermissionStream();
    Q_INVOKABLE void refreshDecision();
    Q_INVOKABLE void refreshAuditLog();
    Q_INVOKABLE void resolveAllowOnce();
    Q_INVOKABLE void resolveAllowForTask();
    Q_INVOKABLE void resolveDeny();

signals:
    void permissionChanged();
    void decisionChanged();
    void auditChanged();
    void errorChanged();
    void busyChanged();

private:
    void handleStreamData(const QByteArray &data);
    void resolve(const QString &decision);
    void clearRequest();
    void fetchDecision(const QString &permissionId);
    void setError(const QString &error);
    void setBusy(bool busy);

    QNetworkAccessManager m_network;
    QString m_streamBuffer;
    QString m_permissionId;
    QString m_permissionLevel;
    QString m_permissionAction;
    QString m_permissionTarget;
    QString m_permissionReason;
    QString m_permissionScope;
    QString m_permissionExpires;
    QString m_lastDecision = QStringLiteral("none");
    QString m_auditText;
    QString m_error;
    bool m_hasPermissionRequest = false;
    bool m_busy = false;
};
