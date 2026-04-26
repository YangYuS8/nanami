#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>
#include <QVector>

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
    struct WorkflowStepView {
        QString kind;
        QString status;
        QString summary;
    };

    struct WorkflowTestResultView {
        QString status;
        QString summary;
        int passed = 0;
        int failed = 0;
    };

    struct WorkflowPatchFileView {
        QString path;
        QString changeType;
        QString diffPreview;
    };

    struct WorkflowPatchView {
        QString patchId;
        QString summary;
        QString diffSummary;
        QVector<WorkflowPatchFileView> files;
    };

    struct WorkflowViewState {
        QString workflowId;
        QString workflowStatus;
        QString projectPath;
        QVector<WorkflowStepView> steps;
        WorkflowTestResultView testResult;
        WorkflowPatchView patch;
    };

    void resetState();
    void handleStreamData(const QByteArray &data);
    void handleEvent(const QJsonObject &event);
    void rebuildDerivedText();
    void setBusy(bool busy);
    void setError(const QString &error);

    QNetworkAccessManager m_network;
    QString m_streamBuffer;
    QString m_stepText;
    QString m_testResultText;
    QString m_patchText;
    QString m_error;
    WorkflowViewState m_state;
    bool m_busy = false;
};
