#pragma once

#include <QObject>
#include <QString>

class PetRendererController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString rendererName READ rendererName NOTIFY rendererChanged)
    Q_PROPERTY(QString rendererStatus READ rendererStatus NOTIFY rendererChanged)
    Q_PROPERTY(QString currentState READ currentState NOTIFY rendererChanged)
    Q_PROPERTY(QString currentEmotion READ currentEmotion NOTIFY rendererChanged)

public:
    explicit PetRendererController(QObject *parent = nullptr);

    QString rendererName() const;
    QString rendererStatus() const;
    QString currentState() const;
    QString currentEmotion() const;

    Q_INVOKABLE void setPersonaState(const QString &state, const QString &emotion);
    Q_INVOKABLE void resetRenderer();

signals:
    void rendererChanged();

private:
    QString m_rendererName = QStringLiteral("Placeholder Renderer");
    QString m_rendererStatus = QStringLiteral("ready");
    QString m_currentState;
    QString m_currentEmotion;
};
