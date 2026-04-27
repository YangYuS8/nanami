cmake_minimum_required(VERSION 3.20)

project(nanami-ui VERSION 0.1.0 LANGUAGES CXX)

option(NANAMI_ENABLE_LIVE2D "Enable optional Cubism SDK for Native build hooks" OFF)
set(NANAMI_CUBISM_SDK_ROOT "" CACHE PATH "Optional path to the local Cubism SDK for Native root")

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Qt6 QUIET COMPONENTS LinguistTools Network Quick Widgets)

if(NOT Qt6_FOUND)
    if(NANAMI_REQUIRE_QT)
        find_package(Qt6 REQUIRED COMPONENTS LinguistTools Network Quick Widgets)
    endif()

    message(WARNING "Qt6 Quick was not found; skipping nanami-ui target")
    return()
endif()

qt_policy(SET QTP0004 NEW)
qt_standard_project_setup(REQUIRES 6.5)

qt_add_executable(nanami-ui
    src/ChatController.cpp
    src/ChatController.h
    src/DesktopController.cpp
    src/DesktopController.h
    src/HttpJsonClient.cpp
    src/HttpJsonClient.h
    src/PersonaController.cpp
    src/PersonaController.h
    src/PetRendererController.cpp
    src/PetRendererController.h
    src/live2d/Live2DRendererAdapter.cpp
    src/live2d/Live2DRendererAdapter.h
    src/ProjectController.cpp
    src/ProjectController.h
    src/PermissionController.cpp
    src/PermissionController.h
    src/SandboxController.cpp
    src/SandboxController.h
    src/StatusController.cpp
    src/StatusController.h
    src/SseStreamParser.cpp
    src/SseStreamParser.h
    src/TaskController.cpp
    src/TaskController.h
    src/WorkflowController.cpp
    src/WorkflowController.h
    src/main.cpp
)

if(NANAMI_ENABLE_LIVE2D)
    if(NANAMI_CUBISM_SDK_ROOT STREQUAL "" OR NOT IS_DIRECTORY "${NANAMI_CUBISM_SDK_ROOT}")
        message(FATAL_ERROR "NANAMI_ENABLE_LIVE2D=ON requires NANAMI_CUBISM_SDK_ROOT to point to a valid local Cubism SDK for Native directory")
    endif()

    target_compile_definitions(nanami-ui PRIVATE NANAMI_ENABLE_LIVE2D)
endif()

qt_add_qml_module(nanami-ui
    URI Nanami
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/PetWindow.qml
        qml/components/CompanionHome.qml
        qml/components/StatusPanel.qml
        qml/components/PetPanel.qml
        qml/components/ChatPanel.qml
        qml/components/TaskPanel.qml
        qml/components/PermissionPanel.qml
        qml/components/SandboxPanel.qml
        qml/components/ProjectPanel.qml
        qml/components/WorkflowPanel.qml
)

qt_add_translations(nanami-ui
    TS_FILES
        i18n/nanami_zh_CN.ts
    RESOURCE_PREFIX "/i18n"
)

target_link_libraries(nanami-ui
    PRIVATE
        Qt6::Network
        Qt6::Quick
        Qt6::Widgets
)
