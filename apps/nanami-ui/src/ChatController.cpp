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
    appendConversation(QStringLiteral("Assistant"), QString());
    m_assistantOpen = true;
    m_streamBuffer.clear();

    QJsonObject body;
    body.insert(QStringLiteral("message"), trimmed);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/chat/stream")));
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    auto *reply = m_network.post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("nanami-core streaming chat endpoint is unavailable"));
            return;
        }
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

void ChatController::appendAssistantDelta(const QString &delta)
{
    if (!m_assistantOpen) {
        appendConversation(QStringLiteral("Assistant"), QString());
        m_assistantOpen = true;
    }

    m_conversationText.append(delta);
    emit conversationTextChanged();
}

void ChatController::handleStreamData(const QByteArray &data)
{
    if (data.isEmpty()) {
        return;
    }

    m_streamBuffer.append(QString::fromUtf8(data));
    int separator = m_streamBuffer.indexOf(QStringLiteral("\n\n"));
    while (separator >= 0) {
        const QString frame = m_streamBuffer.left(separator).trimmed();
        m_streamBuffer.remove(0, separator + 2);

        if (frame.startsWith(QStringLiteral("data:"))) {
            const QString payload = frame.mid(5).trimmed();
            const auto document = QJsonDocument::fromJson(payload.toUtf8());
            if (document.isObject()) {
                handleStreamEvent(document.object());
            }
        }

        separator = m_streamBuffer.indexOf(QStringLiteral("\n\n"));
    }
}

void ChatController::handleStreamEvent(const QJsonObject &event)
{
    const QString kind = event.value(QStringLiteral("kind")).toString();
    if (kind == QStringLiteral("message_delta")) {
        appendAssistantDelta(event.value(QStringLiteral("delta")).toString());
        return;
    }

    if (kind == QStringLiteral("message_completed")) {
        const QString content = event.value(QStringLiteral("content")).toString();
        if (!content.isEmpty() && !m_assistantOpen) {
            appendConversation(QStringLiteral("Assistant"), content);
        }
        m_assistantOpen = false;
        setBusy(false);
        return;
    }

    if (kind == QStringLiteral("error")) {
        const QJsonObject error = event.value(QStringLiteral("error")).toObject();
        setError(error.value(QStringLiteral("message")).toString(QStringLiteral("Chat stream failed")));
        m_assistantOpen = false;
        setBusy(false);
    }
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
