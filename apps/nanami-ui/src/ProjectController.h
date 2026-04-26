#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QString>

class PermissionController;

class ProjectController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString projectId READ projectId NOTIFY projectChanged)
    Q_PROPERTY(QString displayName READ displayName NOTIFY projectChanged)
    Q_PROPERTY(QString projectPath READ projectPath NOTIFY projectChanged)
    Q_PROPERTY(QString projectKind READ projectKind NOTIFY projectChanged)
    Q_PROPERTY(QString trustStatus READ trustStatus NOTIFY projectChanged)
    Q_PROPERTY(QString projectStructureText READ projectStructureText NOTIFY projectChanged)
    Q_PROPERTY(QString manifestPreviewText READ manifestPreviewText NOTIFY projectChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

public:
    explicit ProjectController(QObject *parent = nullptr);

    QString projectId() const;
    QString displayName() const;
    QString projectPath() const;
    QString projectKind() const;
    QString trustStatus() const;
    QString projectStructureText() const;
    QString manifestPreviewText() const;
    bool busy() const;
    QString error() const;

    Q_INVOKABLE void loadMockProject();
    Q_INVOKABLE void selectProjectFolder();
    Q_INVOKABLE void trustSelectedProject();
    Q_INVOKABLE void loadProjectStructure();
    Q_INVOKABLE void requestManifestPreviewPermission();
    Q_INVOKABLE void loadManifestPreview();
    void setPermissionController(PermissionController *permissionController);

signals:
    void projectChanged();
    void busyChanged();
    void errorChanged();

private:
    void setBusy(bool busy);
    void setError(const QString &error);

    QNetworkAccessManager m_network;
    PermissionController *m_permissionController = nullptr;
    QString m_projectId;
    QString m_displayName;
    QString m_projectPath;
    QString m_projectKind;
    QString m_trustStatus;
    QString m_projectStructureText;
    QString m_manifestPreviewText;
    QString m_error;
    bool m_busy = false;
};
