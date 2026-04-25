#include "ChatController.h"

#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

ChatController::ChatController(QObject *parent)
    : QObject(parent)
{
}

QString ChatController::conversationText() const
{
    return m_conversationText;
}

QString ChatController::error() const
{
    return m_error;
}

bool ChatController::busy() const
{
    return m_busy;
}

void ChatController::sendMessage(const QString &text)
{
    const QString trimmed = text.trimmed();
    if (trimmed.isEmpty() || m_busy) {
        return;
    }

    setError(QString());
    setBusy(true);
    appendConversation(QStringLiteral("User"), trimmed);

    QJsonObject body;
    body.insert(QStringLiteral("message"), trimmed);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/chat")));
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    auto *reply = m_network.post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (document.isObject()) {
            const auto object = document.object();
            const QString content = object.value(QStringLiteral("content")).toString();
            if (!content.isEmpty()) {
                appendConversation(QStringLiteral("Assistant"), content);
                return;
            }

            const QString message = object.value(QStringLiteral("message")).toString();
            if (!message.isEmpty()) {
                setError(message);
                return;
            }
        }

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("nanami-core chat endpoint is unavailable"));
            return;
        }

        if (!document.isObject()) {
            setError(QStringLiteral("Invalid chat response"));
            return;
        }

        setError(QStringLiteral("Chat request failed"));
    });
}

void ChatController::appendConversation(const QString &speaker, const QString &message)
{
    if (!m_conversationText.isEmpty()) {
        m_conversationText.append(QStringLiteral("\n\n"));
    }

    m_conversationText.append(speaker + QStringLiteral(": ") + message);
    emit conversationTextChanged();
}

void ChatController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}

void ChatController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}
