#include "PersonaController.h"

#include "HttpJsonClient.h"
#include "SseStreamParser.h"

#include <QJsonObject>
#include <QNetworkReply>
#include <QUrl>

PersonaController::PersonaController(QObject *parent)
    : QObject(parent)
{
}

QString PersonaController::state() const
{
    return m_state;
}

QString PersonaController::emotion() const
{
    return m_emotion;
}

QString PersonaController::text() const
{
    return m_text;
}

QString PersonaController::source() const
{
    return m_source;
}

bool PersonaController::busy() const
{
    return m_busy;
}

QString PersonaController::error() const
{
    return m_error;
}

void PersonaController::startMockPersonaStream()
{
    if (m_busy) {
        return;
    }

    resetState();
    m_streamBuffer.clear();
    setError(QString());
    setBusy(true);

    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/persona/mock/stream")));

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("nanami-core mock persona stream is unavailable")));
        }
    });
}

void PersonaController::resetState()
{
    m_state.clear();
    m_emotion.clear();
    m_text.clear();
    m_source.clear();
    emit personaChanged();
}

void PersonaController::handleStreamData(const QByteArray &data)
{
    if (data.isEmpty()) {
        return;
    }

    const QStringList payloads = SseStreamParser::extractDataFrames(&m_streamBuffer, data);
    for (const QString &payload : payloads) {
        QJsonObject object;
        const auto document = QJsonDocument::fromJson(payload.toUtf8());
        if (document.isObject()) {
            object = document.object();
            handleEvent(object);
        }
    }
}

void PersonaController::handleEvent(const QJsonObject &event)
{
    if (event.value(QStringLiteral("type")).toString() == QStringLiteral("persona.state")) {
        m_state = event.value(QStringLiteral("state")).toString();
        m_emotion = event.value(QStringLiteral("emotion")).toString();
        m_text = event.value(QStringLiteral("text")).toString();
        m_source = event.value(QStringLiteral("source")).toString();
        emit personaChanged();
        return;
    }

    if (event.value(QStringLiteral("type")).toString() == QStringLiteral("error.occurred")) {
        setError(event.value(QStringLiteral("message")).toString(QStringLiteral("Mock persona stream failed")));
    }
}

void PersonaController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}

void PersonaController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}
