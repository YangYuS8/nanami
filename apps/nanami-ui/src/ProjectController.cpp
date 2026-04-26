#include "ProjectController.h"

#include <QFileDialog>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
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

QString ProjectController::projectStructureText() const
{
    return m_projectStructureText;
}

QString ProjectController::manifestPreviewText() const
{
    return m_manifestPreviewText;
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
        m_projectStructureText.clear();
        m_manifestPreviewText.clear();
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
        m_projectStructureText.clear();
        m_manifestPreviewText.clear();
        emit projectChanged();
    });
}

void ProjectController::trustSelectedProject()
{
    if (m_busy || m_projectId.isEmpty()) {
        return;
    }

    setError(QString());
    setBusy(true);

    QJsonObject body;
    body.insert(QStringLiteral("project_id"), m_projectId);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/trust")));
    request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("application/json"));
    auto *reply = m_network.post(request, QJsonDocument(body).toJson(QJsonDocument::Compact));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("Failed to trust selected project"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setError(QStringLiteral("Invalid trusted project response"));
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

void ProjectController::loadProjectStructure()
{
    if (m_busy) {
        return;
    }

    setError(QString());
    setBusy(true);

    QNetworkRequest request(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/current/structure")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("Failed to load project structure"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setError(QStringLiteral("Invalid project structure response"));
            return;
        }

        QStringList lines;
        const auto entries = document.object().value(QStringLiteral("entries")).toArray();
        for (const auto &value : entries) {
            const auto entry = value.toObject();
            lines.append(QStringLiteral("%1 [%2, %3]")
                             .arg(entry.value(QStringLiteral("relative_path")).toString(),
                                  entry.value(QStringLiteral("entry_type")).toString(),
                                  entry.value(QStringLiteral("marker")).toString()));
        }
        m_projectStructureText = lines.join(QStringLiteral("\n"));
        emit projectChanged();
    });
}

void ProjectController::requestManifestPreviewPermission()
{
    if (m_busy) {
        return;
    }

    setError(QString());
    setBusy(true);

    QNetworkRequest request(
        QUrl(QStringLiteral("http://127.0.0.1:17878/projects/current/manifest/preview-request")));
    auto *reply = m_network.post(request, QByteArray());

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("Failed to request manifest preview permission"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setError(QStringLiteral("Invalid manifest preview permission response"));
            return;
        }

        const auto object = document.object();
        m_manifestPreviewText = QStringLiteral("Permission requested: %1\nAction: %2\nTarget: %3")
                                   .arg(object.value(QStringLiteral("permission_id")).toString(),
                                        object.value(QStringLiteral("action")).toString(),
                                        object.value(QStringLiteral("target")).toString());
        emit projectChanged();
    });
}

void ProjectController::loadManifestPreview()
{
    if (m_busy) {
        return;
    }

    setError(QString());
    setBusy(true);

    QNetworkRequest request(
        QUrl(QStringLiteral("http://127.0.0.1:17878/projects/current/manifest/preview")));
    auto *reply = m_network.get(request);

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(QStringLiteral("Failed to load manifest preview"));
            return;
        }

        const auto document = QJsonDocument::fromJson(reply->readAll());
        if (!document.isObject()) {
            setError(QStringLiteral("Invalid manifest preview response"));
            return;
        }

        const auto object = document.object();
        const QString header = QStringLiteral("Manifest: %1\nKind: %2\nSize: %3 bytes%4\n")
                                   .arg(object.value(QStringLiteral("manifest_path")).toString(),
                                        object.value(QStringLiteral("kind")).toString(),
                                        QString::number(object.value(QStringLiteral("size_bytes")).toInteger()),
                                        object.value(QStringLiteral("truncated")).toBool()
                                            ? QStringLiteral(" (truncated)")
                                            : QString());
        m_manifestPreviewText = header + QStringLiteral("\n")
            + object.value(QStringLiteral("content_preview")).toString();
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
