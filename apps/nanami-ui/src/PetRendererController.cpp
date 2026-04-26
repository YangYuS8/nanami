#include "PetRendererController.h"

PetRendererController::PetRendererController(QObject *parent)
    : QObject(parent)
{
    applyPlaceholderBackend();
}

QString PetRendererController::rendererName() const
{
    return m_rendererName;
}

QString PetRendererController::rendererStatus() const
{
    return m_rendererStatus;
}

QString PetRendererController::rendererBackend() const
{
    return m_rendererBackend;
}

QString PetRendererController::rendererAvailability() const
{
    return m_rendererAvailability;
}

QString PetRendererController::modelPath() const
{
    return m_modelPath;
}

bool PetRendererController::modelLoaded() const
{
    return m_modelLoaded;
}

QString PetRendererController::lastRendererError() const
{
    return m_lastRendererError;
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
    if (m_rendererBackend == tr("live2d")) {
        nextStatus = m_modelLoaded ? tr("live2d_active") : tr("live2d_selected");
    } else if (state.isEmpty() && emotion.isEmpty()) {
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
    if (m_currentState.isEmpty() && m_currentEmotion.isEmpty()) {
        return;
    }

    m_currentState.clear();
    m_currentEmotion.clear();
    m_rendererStatus = m_rendererBackend == tr("live2d")
        ? (m_modelLoaded ? tr("live2d_ready") : tr("live2d_selected"))
        : tr("ready");
    emit rendererChanged();
}

void PetRendererController::selectPlaceholderRenderer()
{
    const QString preservedModelPath = m_modelPath;
    applyPlaceholderBackend();
    m_modelPath = preservedModelPath;
    emit rendererChanged();
}

void PetRendererController::selectLive2DRenderer()
{
    applyLive2DBackendIntent();
    emit rendererChanged();
}

void PetRendererController::setModelPath(const QString &path)
{
    const QString trimmed = path.trimmed();
    if (m_modelPath == trimmed) {
        return;
    }

    m_modelPath = trimmed;
    emit rendererChanged();
}

void PetRendererController::loadModel()
{
    if (m_rendererBackend == tr("placeholder")) {
        m_modelLoaded = false;
        m_lastRendererError = tr("Placeholder renderer does not load external models");
        m_rendererStatus = tr("placeholder_selected");
        emit rendererChanged();
        return;
    }

    if (m_modelPath.isEmpty()) {
        m_modelLoaded = false;
        m_lastRendererError = tr("Live2D model path is not configured");
        m_rendererStatus = tr("live2d_unavailable");
        emit rendererChanged();
        return;
    }

    m_modelLoaded = false;
    m_lastRendererError = tr("Live2D SDK is unavailable in this build");
    m_rendererStatus = tr("live2d_unavailable");
    emit rendererChanged();
}

void PetRendererController::unloadModel()
{
    if (!m_modelLoaded && m_lastRendererError.isEmpty()) {
        return;
    }

    m_modelLoaded = false;
    clearRendererError();
    m_rendererStatus = m_rendererBackend == tr("live2d") ? tr("live2d_selected") : tr("ready");
    emit rendererChanged();
}

void PetRendererController::applyPlaceholderBackend()
{
    m_rendererName = tr("Placeholder Renderer");
    m_rendererBackend = tr("placeholder");
    m_rendererAvailability = tr("available");
    m_rendererStatus = tr("ready");
    m_modelLoaded = false;
    clearRendererError();
}

void PetRendererController::applyLive2DBackendIntent()
{
    m_rendererName = tr("Live2D Renderer");
    m_rendererBackend = tr("live2d");
    m_rendererAvailability = tr("unavailable");
    m_rendererStatus = tr("live2d_selected");
    m_modelLoaded = false;
    clearRendererError();
}

void PetRendererController::clearRendererError()
{
    m_lastRendererError.clear();
    emit rendererChanged();
}
