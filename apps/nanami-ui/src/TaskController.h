#pragma once

#include <QNetworkAccessManager>
#include <QHash>
#include <QObject>
#include <QString>
#include <QStringList>
#include <QVector>

class TaskController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString taskTimelineText READ taskTimelineText NOTIFY taskTimelineTextChanged)
    Q_PROPERTY(QString currentTaskId READ currentTaskId NOTIFY currentTaskChanged)
    Q_PROPERTY(QString currentTaskStatus READ currentTaskStatus NOTIFY currentTaskChanged)
    Q_PROPERTY(QString currentTaskTitle READ currentTaskTitle NOTIFY currentTaskChanged)
    Q_PROPERTY(int toolCount READ toolCount NOTIFY currentTaskChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)

public:
    explicit TaskController(QObject *parent = nullptr);

    QString taskTimelineText() const;
    QString currentTaskId() const;
    QString currentTaskStatus() const;
    QString currentTaskTitle() const;
    int toolCount() const;
    QString error() const;
    bool busy() const;
    Q_INVOKABLE void startMockTaskStream();
    Q_INVOKABLE void startOpenClawTaskStream(const QString &message);

signals:
    void taskTimelineTextChanged();
    void currentTaskChanged();
    void errorChanged();
    void busyChanged();

private:
    struct ToolOutputView {
        QString stream;
        QString content;
    };

    struct ToolViewState {
        QString toolCallId;
        QString tool;
        QString status;
        QString exitCode;
        QVector<ToolOutputView> outputs;
    };

    struct TaskViewState {
        QString taskId;
        QString title;
        QString status;
        QString summary;
        QHash<QString, ToolViewState> tools;
        QVector<QString> toolOrder;
    };

    void resetState();
    void rebuildTimeline();
    void handleStreamData(const QByteArray &data);
    void handleEvent(const QJsonObject &event);
    void setError(const QString &error);
    void setBusy(bool busy);

    QNetworkAccessManager m_network;
    QString m_taskTimelineText;
    QString m_streamBuffer;
    QString m_error;
    QStringList m_permissionLines;
    QStringList m_activityLines;
    TaskViewState m_currentTask;
    bool m_busy = false;
};
