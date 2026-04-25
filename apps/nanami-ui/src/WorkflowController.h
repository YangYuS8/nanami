#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class WorkflowController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString workflowId READ workflowId NOTIFY workflowChanged)
    Q_PROPERTY(QString workflowStatus READ workflowStatus NOTIFY workflowChanged)
    Q_PROPERTY(QString projectPath READ projectPath NOTIFY workflowChanged)
    Q_PROPERTY(QString stepText READ stepText NOTIFY workflowChanged)
    Q_PROPERTY(QString testResultText READ testResultText NOTIFY workflowChanged)
    Q_PROPERTY(QString patchText READ patchText NOTIFY workflowChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

public:
    explicit WorkflowController(QObject *parent = nullptr);

    QString workflowId() const;
    QString workflowStatus() const;
    QString projectPath() const;
    QString stepText() const;
    QString testResultText() const;
    QString patchText() const;
    bool busy() const;
    QString error() const;

    Q_INVOKABLE void startMockWorkflowStream();

signals:
    void workflowChanged();
    void busyChanged();
    void errorChanged();

private:
    void resetState();
    void handleStreamData(const QByteArray &data);
    void handleEvent(const QJsonObject &event);
    void setBusy(bool busy);
    void setError(const QString &error);

    QNetworkAccessManager m_network;
    QString m_streamBuffer;
    QString m_workflowId;
    QString m_workflowStatus;
    QString m_projectPath;
    QString m_stepText;
    QString m_testResultText;
    QString m_patchText;
    QString m_error;
    bool m_busy = false;
};
