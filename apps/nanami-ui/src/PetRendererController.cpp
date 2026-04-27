#include "PetRendererController.h"

#include "live2d/Live2DRendererAdapter.h"

namespace {
constexpr auto kBackendPlaceholder = "placeholder";
constexpr auto kBackendLive2D = "live2d";
constexpr auto kAvailabilityAvailable = "available";
constexpr auto kAvailabilityUnavailable = "unavailable";
constexpr auto kStatusReady = "ready";
constexpr auto kStatusPlaceholderActive = "placeholder_active";
constexpr auto kStatusPlaceholderSelected = "placeholder_selected";
constexpr auto kStatusLive2DSelected = "live2d_selected";
constexpr auto kStatusLive2DUnavailable = "live2d_unavailable";
constexpr auto kStatusLive2DReady = "live2d_ready";
constexpr auto kStatusLive2DActive = "live2d_active";
}

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
    QString nextStatus = QString::fromLatin1(kStatusPlaceholderActive);
    if (m_rendererBackend == QLatin1String(kBackendLive2D)) {
        nextStatus = m_modelLoaded ? QString::fromLatin1(kStatusLive2DActive)
                                   : QString::fromLatin1(kStatusLive2DSelected);
    } else if (state.isEmpty() && emotion.isEmpty()) {
        nextStatus = QString::fromLatin1(kStatusReady);
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
    m_rendererStatus = m_rendererBackend == QLatin1String(kBackendLive2D)
        ? (m_modelLoaded ? QString::fromLatin1(kStatusLive2DReady)
                         : QString::fromLatin1(kStatusLive2DSelected))
        : QString::fromLatin1(kStatusReady);
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

    if (!Live2DRendererAdapter::isBuiltWithLive2DSupport()
        || !Live2DRendererAdapter::isRuntimeAvailable()) {
        m_rendererAvailability = QString::fromLatin1(kAvailabilityUnavailable);
        m_lastRendererError = Live2DRendererAdapter::unavailableReason();
    }

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
    if (m_rendererBackend == QLatin1String(kBackendPlaceholder)) {
        m_modelLoaded = false;
        m_lastRendererError = tr("Placeholder renderer does not load external models");
        m_rendererStatus = QString::fromLatin1(kStatusPlaceholderSelected);
        emit rendererChanged();
        return;
    }

    if (!Live2DRendererAdapter::isBuiltWithLive2DSupport()
        || !Live2DRendererAdapter::isRuntimeAvailable()) {
        m_modelLoaded = false;
        m_lastRendererError = Live2DRendererAdapter::unavailableReason();
        m_rendererAvailability = QString::fromLatin1(kAvailabilityUnavailable);
        m_rendererStatus = QString::fromLatin1(kStatusLive2DUnavailable);
        emit rendererChanged();
        return;
    }

    if (m_modelPath.isEmpty()) {
        m_modelLoaded = false;
        m_lastRendererError = tr("Live2D model path is not configured");
        m_rendererStatus = QString::fromLatin1(kStatusLive2DUnavailable);
        emit rendererChanged();
        return;
    }

    m_modelLoaded = false;
    m_lastRendererError = tr("Live2D SDK is unavailable in this build");
    m_rendererStatus = QString::fromLatin1(kStatusLive2DUnavailable);
    emit rendererChanged();
}

void PetRendererController::unloadModel()
{
    if (!m_modelLoaded && m_lastRendererError.isEmpty()) {
        return;
    }

    m_modelLoaded = false;
    clearRendererError();
    m_rendererStatus = m_rendererBackend == QLatin1String(kBackendLive2D)
        ? QString::fromLatin1(kStatusLive2DSelected)
        : QString::fromLatin1(kStatusReady);
    emit rendererChanged();
}

void PetRendererController::applyPlaceholderBackend()
{
    m_rendererName = tr("Placeholder Renderer");
    m_rendererBackend = QString::fromLatin1(kBackendPlaceholder);
    m_rendererAvailability = QString::fromLatin1(kAvailabilityAvailable);
    m_rendererStatus = QString::fromLatin1(kStatusReady);
    m_modelLoaded = false;
    clearRendererError();
}

void PetRendererController::applyLive2DBackendIntent()
{
    m_rendererName = tr("Live2D Renderer");
    m_rendererBackend = QString::fromLatin1(kBackendLive2D);
    m_rendererAvailability = QString::fromLatin1(kAvailabilityUnavailable);
    m_rendererStatus = QString::fromLatin1(kStatusLive2DSelected);
    m_modelLoaded = false;
    clearRendererError();
}

void PetRendererController::clearRendererError()
{
    m_lastRendererError.clear();
    emit rendererChanged();
}
