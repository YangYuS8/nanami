#include "Live2DRendererAdapter.h"

#include <QCoreApplication>

bool Live2DRendererAdapter::isBuiltWithLive2DSupport()
{
#ifdef NANAMI_ENABLE_LIVE2D
    return true;
#else
    return false;
#endif
}

bool Live2DRendererAdapter::isRuntimeAvailable()
{
    return false;
}

QString Live2DRendererAdapter::unavailableReason()
{
#ifdef NANAMI_ENABLE_LIVE2D
    return QCoreApplication::translate(
        "Live2DRendererAdapter",
        "Live2D SDK integration is not implemented in this build");
#else
    return QCoreApplication::translate(
        "Live2DRendererAdapter",
        "Live2D support is not enabled in this build");
#endif
}
