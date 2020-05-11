cmake_policy(SET CMP0091 NEW)

project(scissors)

# set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDebugDLL")

find_package(Qt5 COMPONENTS Widgets REQUIRED)

add_library(cscissors STATIC wrapper_c.c wrapper.cc)

target_link_libraries(cscissors Qt5::Widget)
target_include_directories(cscissors PRIVATE ${CMAKE_PREFIX_PATH}/include ${CMAKE_PREFIX_PATH}/include/QtWidgets ${CMAKE_PREFIX_PATH}/include/QtCore ${CMAKE_CURRENT_SOURCE_DIR}/obs-studio/libobs)