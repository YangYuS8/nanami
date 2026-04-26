#include "ChatController.h"

#include "HttpJsonClient.h"
#include "SseStreamParser.h"

#include <QJsonObject>
#include <QNetworkReply>
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
    appendConversation(tr("User"), trimmed);
    appendConversation(tr("Assistant"), QString());
    m_assistantOpen = true;
    m_assistantHasContent = false;
    m_streamBuffer.clear();

    QJsonObject body;
    body.insert(QStringLiteral("message"), trimmed);

    HttpJsonClient client(&m_network);
    auto *reply = client.postJson(QUrl(QStringLiteral("http://127.0.0.1:17878/chat/stream")), body);

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("nanami-core streaming chat endpoint is unavailable")));
            m_assistantOpen = false;
            m_assistantHasContent = false;
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
        appendConversation(tr("Assistant"), QString());
        m_assistantOpen = true;
        m_assistantHasContent = false;
    }

    m_conversationText.append(delta);
    m_assistantHasContent = true;
    emit conversationTextChanged();
}

void ChatController::handleStreamData(const QByteArray &data)
{
    if (data.isEmpty()) {
        return;
    }

    const QStringList payloads = SseStreamParser::extractDataFrames(&m_streamBuffer, data);
    for (const QString &payload : payloads) {
        const auto document = QJsonDocument::fromJson(payload.toUtf8());
        if (document.isObject()) {
            handleStreamEvent(document.object());
        }
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
        if (!content.isEmpty() && !m_assistantHasContent) {
            appendAssistantDelta(content);
        }
        m_assistantOpen = false;
        m_assistantHasContent = false;
        setBusy(false);
        return;
    }

    if (kind == QStringLiteral("error")) {
        const QJsonObject error = event.value(QStringLiteral("error")).toObject();
        setError(error.value(QStringLiteral("message")).toString(tr("Chat stream failed")));
        m_assistantOpen = false;
        m_assistantHasContent = false;
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
