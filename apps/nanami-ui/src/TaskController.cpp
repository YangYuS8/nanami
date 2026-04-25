#include "TaskController.h"

#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

TaskController::TaskController(QObject *parent)
    : QObject(parent)
{
}

QString TaskController::taskTimelineText() const
{
    return m_taskTimelineText;
}

QString TaskController::error() const
{
    return m_error;
}

bool TaskController::busy() const
{
    return m_busy;
}

void TaskController::startMockTaskStream()
{
    if (m_busy) {
        return;
    }

    m_streamBuffer.clear();
    m_taskTimelineText.clear();
    emit taskTimelineTextChanged();
    setError(QString());
    setBusy(true);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/tasks/mock/stream")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::readyRead, this, [this, reply]() {
        handleStreamData(reply->readAll());
    });

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        handleStreamData(reply->readAll());
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("nanami-core mock task stream is unavailable"));
        }
    });
}

void TaskController::startOpenClawTaskStream(const QString &message)
{
    const QString trimmed = message.trimmed();
    if (trimmed.isEmpty() || m_busy) {
        return;
    }

    m_streamBuffer.clear();
    m_taskTimelineText.clear();
    emit taskTimelineTextChanged();
    setError(QString());
    setBusy(true);

    QJsonObject body;
    body.insert(QStringLiteral("message"), trimmed);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/tasks/openclaw/stream")));
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
            setError(QStringLiteral("nanami-core OpenClaw task stream is unavailable"));
        }
    });
}

void TaskController::appendTimeline(const QString &line)
{
    if (!m_taskTimelineText.isEmpty()) {
        m_taskTimelineText.append(QStringLiteral("\n"));
    }

    m_taskTimelineText.append(line);
    emit taskTimelineTextChanged();
}

void TaskController::handleStreamData(const QByteArray &data)
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
                handleEvent(document.object());
            }
        }

        separator = m_streamBuffer.indexOf(QStringLiteral("\n\n"));
    }
}

void TaskController::handleEvent(const QJsonObject &event)
{
    const QString type = event.value(QStringLiteral("type")).toString();

    if (type == QStringLiteral("task.started")) {
        appendTimeline(QStringLiteral("Task %1 started: %2")
                           .arg(event.value(QStringLiteral("task_id")).toString(),
                                event.value(QStringLiteral("title")).toString()));
        return;
    }

    if (type == QStringLiteral("tool.started")) {
        appendTimeline(QStringLiteral("Tool %1 started: %2")
                           .arg(event.value(QStringLiteral("tool_call_id")).toString(),
                                event.value(QStringLiteral("tool")).toString()));
        return;
    }

    if (type == QStringLiteral("tool.output")) {
        appendTimeline(QStringLiteral("%1: %2")
                           .arg(event.value(QStringLiteral("stream")).toString(),
                                event.value(QStringLiteral("content")).toString()));
        return;
    }

    if (type == QStringLiteral("tool.completed")) {
        appendTimeline(QStringLiteral("Tool %1 completed: status=%2")
                           .arg(event.value(QStringLiteral("tool_call_id")).toString(),
                                event.value(QStringLiteral("status")).toString()));
        return;
    }

    if (type == QStringLiteral("task.completed")) {
        appendTimeline(QStringLiteral("Task %1 completed: %2")
                           .arg(event.value(QStringLiteral("task_id")).toString(),
                                event.value(QStringLiteral("summary")).toString()));
        return;
    }

    if (type == QStringLiteral("error.occurred")) {
        setError(event.value(QStringLiteral("message")).toString(QStringLiteral("OpenClaw task stream failed")));
    }
}

void TaskController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}

void TaskController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}
