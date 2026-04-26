#include "PetRendererController.h"

PetRendererController::PetRendererController(QObject *parent)
    : QObject(parent)
{
    m_rendererName = tr("Placeholder Renderer");
    m_rendererStatus = tr("ready");
}

QString PetRendererController::rendererName() const
{
    return m_rendererName;
}

QString PetRendererController::rendererStatus() const
{
    return m_rendererStatus;
}

QString PetRendererController::currentState() const
{
    return m_currentState;
}

QString PetRendererController::currentEmotion() const
{
    return m_currentEmotion;
}

void PetRendererController::setPersonaState(const QString &state, const QString &emotion)
{
    QString nextStatus = tr("placeholder_active");
    if (state.isEmpty() && emotion.isEmpty()) {
        nextStatus = tr("ready");
    }

    if (m_currentState == state && m_currentEmotion == emotion && m_rendererStatus == nextStatus) {
        return;
    }

    m_currentState = state;
    m_currentEmotion = emotion;
    m_rendererStatus = nextStatus;
    emit rendererChanged();
}

void PetRendererController::resetRenderer()
{
    if (m_currentState.isEmpty() && m_currentEmotion.isEmpty()
        && m_rendererStatus == tr("ready")) {
        return;
    }

    m_currentState.clear();
    m_currentEmotion.clear();
    m_rendererStatus = tr("ready");
    emit rendererChanged();
}
