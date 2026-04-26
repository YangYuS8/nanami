#include "ProjectController.h"

#include "HttpJsonClient.h"
#include "PermissionController.h"

#include <QFileDialog>
#include <QJsonArray>
#include <QJsonObject>
#include <QNetworkReply>
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

QString ProjectController::manifestSummaryText() const
{
    return m_manifestSummaryText;
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

    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/mock/current")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("nanami-core mock project endpoint is unavailable")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(QStringLiteral("Invalid mock project response"));
            return;
        }
        m_projectId = object.value(QStringLiteral("project_id")).toString();
        m_displayName = object.value(QStringLiteral("display_name")).toString();
        m_projectPath = object.value(QStringLiteral("project_path")).toString();
        m_projectKind = object.value(QStringLiteral("kind")).toString();
        m_trustStatus = object.value(QStringLiteral("trust_status")).toString();
        m_projectStructureText.clear();
        m_manifestPreviewText.clear();
        m_manifestSummaryText.clear();
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

    HttpJsonClient client(&m_network);
    auto *reply = client.postJson(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/select")), body);

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("Failed to select project folder")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(QStringLiteral("Invalid selected project response"));
            return;
        }
        m_projectId = object.value(QStringLiteral("project_id")).toString();
        m_displayName = object.value(QStringLiteral("display_name")).toString();
        m_projectPath = object.value(QStringLiteral("project_path")).toString();
        m_projectKind = object.value(QStringLiteral("kind")).toString();
        m_trustStatus = object.value(QStringLiteral("trust_status")).toString();
        m_projectStructureText.clear();
        m_manifestPreviewText.clear();
        m_manifestSummaryText.clear();
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

    HttpJsonClient client(&m_network);
    auto *reply = client.postJson(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/trust")), body);

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("Failed to trust selected project")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(QStringLiteral("Invalid trusted project response"));
            return;
        }
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

    HttpJsonClient client(&m_network);
    auto *reply = client.get(QUrl(QStringLiteral("http://127.0.0.1:17878/projects/current/structure")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("Failed to load project structure")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(QStringLiteral("Invalid project structure response"));
            return;
        }

        QStringList lines;
        const auto entries = object.value(QStringLiteral("entries")).toArray();
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

    HttpJsonClient client(&m_network);
    auto *reply = client.postEmpty(
        QUrl(QStringLiteral("http://127.0.0.1:17878/projects/current/manifest/preview-request")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("Failed to request manifest preview permission")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(QStringLiteral("Invalid manifest preview permission response"));
            return;
        }
        if (m_permissionController != nullptr) {
            m_permissionController->acceptPermissionRequest(object);
        }
        m_manifestPreviewText = QStringLiteral(
                                   "Manifest preview permission requested.\nPermission ID: %1\nApprove or deny it in the Permission Request panel before loading the preview.")
                                   .arg(object.value(QStringLiteral("permission_id")).toString());
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

    HttpJsonClient client(&m_network);
    auto *reply = client.get(
        QUrl(QStringLiteral("http://127.0.0.1:17878/projects/current/manifest/preview")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("Failed to load manifest preview")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(QStringLiteral("Invalid manifest preview response"));
            return;
        }
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

void ProjectController::loadManifestSummary()
{
    if (m_busy) {
        return;
    }

    setError(QString());
    setBusy(true);

    HttpJsonClient client(&m_network);
    auto *reply = client.get(
        QUrl(QStringLiteral("http://127.0.0.1:17878/projects/current/manifest/summary")));

    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        reply->deleteLater();
        setBusy(false);

        if (reply->error() != QNetworkReply::NoError) {
            setError(HttpJsonClient::networkErrorString(
                reply, QStringLiteral("Failed to load manifest summary")));
            return;
        }

        QJsonObject object;
        QString parseError;
        if (!HttpJsonClient::parseObject(reply, &object, &parseError)) {
            setError(QStringLiteral("Invalid manifest summary response"));
            return;
        }
        QStringList lines;
        lines.append(QStringLiteral("Manifest: %1")
                         .arg(object.value(QStringLiteral("manifest_path")).toString()));
        lines.append(
            QStringLiteral("Kind: %1").arg(object.value(QStringLiteral("kind")).toString()));
        lines.append(QStringLiteral("Package: %1")
                         .arg(object.value(QStringLiteral("package_name")).toString(
                             QStringLiteral("unknown"))));
        lines.append(QStringLiteral("Version: %1")
                         .arg(object.value(QStringLiteral("package_version")).toString(
                             QStringLiteral("unknown"))));
        lines.append(QStringLiteral("Dependencies: %1")
                         .arg(object.value(QStringLiteral("dependency_count")).isNull()
                                  ? QStringLiteral("unknown")
                                  : QString::number(
                                        object.value(QStringLiteral("dependency_count")).toInteger())));
        lines.append(QStringLiteral("Scripts: %1")
                         .arg(object.value(QStringLiteral("script_count")).isNull()
                                  ? QStringLiteral("unknown")
                                  : QString::number(
                                        object.value(QStringLiteral("script_count")).toInteger())));
        lines.append(QStringLiteral("Summary: %1")
                         .arg(object.value(QStringLiteral("summary_text")).toString()));
        m_manifestSummaryText = lines.join(QStringLiteral("\n"));
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

void ProjectController::setPermissionController(PermissionController *permissionController)
{
    m_permissionController = permissionController;
}

void ProjectController::setError(const QString &error)
{
    if (m_error == error) {
        return;
    }

    m_error = error;
    emit errorChanged();
}
