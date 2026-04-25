#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class ChatController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString conversationText READ conversationText NOTIFY conversationTextChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)

public:
    explicit ChatController(QObject *parent = nullptr);

    QString conversationText() const;
    QString error() const;
    bool busy() const;
    Q_INVOKABLE void sendMessage(const QString &text);

signals:
    void conversationTextChanged();
    void errorChanged();
    void busyChanged();

private:
    void appendConversation(const QString &speaker, const QString &message);
    void setError(const QString &error);
    void setBusy(bool busy);

    QNetworkAccessManager m_network;
    QString m_conversationText;
    QString m_error;
    bool m_busy = false;
};
