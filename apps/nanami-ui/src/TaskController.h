#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class TaskController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString taskTimelineText READ taskTimelineText NOTIFY taskTimelineTextChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)

public:
    explicit TaskController(QObject *parent = nullptr);

    QString taskTimelineText() const;
    QString error() const;
    bool busy() const;
    Q_INVOKABLE void startMockTaskStream();

signals:
    void taskTimelineTextChanged();
    void errorChanged();
    void busyChanged();

private:
    void appendTimeline(const QString &line);
    void handleStreamData(const QByteArray &data);
    void handleEvent(const QJsonObject &event);
    void setError(const QString &error);
    void setBusy(bool busy);

    QNetworkAccessManager m_network;
    QString m_taskTimelineText;
    QString m_streamBuffer;
    QString m_error;
    bool m_busy = false;
};
