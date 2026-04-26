#include "ProjectController.h"

#include <QJsonDocument>
#include <QJsonObject>
#include <QFileDialog>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QUrl>

ProjectController::ProjectController(QObject *parent)
    : QObject(parent)
{
}

QString ProjectController::projectId() const
{
    return m_projectId;
}

QString ProjectController::displayName() const
{
    return m_displayName;
}

QString ProjectController::projectPath() const
{
    return m_projectPath;
}

QString ProjectController::projectKind() const
{
    return m_projectKind;
}

QString ProjectController::trustStatus() const
{
    return m_trustStatus;
}

bool ProjectController::busy() const
{
    return m_busy;
}

QString ProjectController::error() const
{
    return m_error;
}

void ProjectController::loadMockProject()
{
    if (m_busy) {
        return;
    }

    setError(QString());
    setBusy(true);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/mock/current")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("nanami-core mock project endpoint is unavailable"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setError(QStringLiteral("Invalid mock project response"));
            return;
        }

        const auto object = document.object();
        m_projectId = object.value(QStringLiteral("project_id")).toString();
        m_displayName = object.value(QStringLiteral("display_name")).toString();
        m_projectPath = object.value(QStringLiteral("project_path")).toString();
        m_projectKind = object.value(QStringLiteral("kind")).toString();
        m_trustStatus = object.value(QStringLiteral("trust_status")).toString();
        emit projectChanged();
    });
}

void ProjectController::selectProjectFolder()
{
    if (m_busy) {
        return;
    }

    const QString selectedPath = QFileDialog::getExistingDirectory(
        nullptr,
        QStringLiteral("Select Project Folder"),
        m_projectPath.isEmpty() ? QString() : m_projectPath);
    if (selectedPath.isEmpty()) {
        return;
    }

    setError(QString());
    setBusy(true);

    QJsonObject body;
    body.insert(QStringLiteral("project_path"), selectedPath);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/select")));
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    auto *reply = m_network.post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("Failed to select project folder"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setError(QStringLiteral("Invalid selected project response"));
            return;
        }

        const auto object = document.object();
        m_projectId = object.value(QStringLiteral("project_id")).toString();
        m_displayName = object.value(QStringLiteral("display_name")).toString();
        m_projectPath = object.value(QStringLiteral("project_path")).toString();
        m_projectKind = object.value(QStringLiteral("kind")).toString();
        m_trustStatus = object.value(QStringLiteral("trust_status")).toString();
        emit projectChanged();
    });
}

void ProjectController::setBusy(bool busy)
{
    if (m_busy == busy) {
        return;
    }

    m_busy = busy;
    emit busyChanged();
}

void ProjectController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}
