cmake_minimum_required(VERSION 4.0)
project(mootest)

set(CMAKE_CXX_STANDARD 11)

include_directories(.)

option(USE_ZLIB "Enable zlib support" ON)

add_executable(mootest
        mooreader.h
        mootest.cpp)

if(USE_ZLIB)
    find_package(ZLIB)
    if(ZLIB_FOUND)
        message(STATUS "ZLIB found: ${ZLIB_VERSION}")
        target_compile_definitions(mootest PRIVATE MOO_USE_ZLIB)
        target_link_libraries(mootest PRIVATE ZLIB::ZLIB)
    else()
        message(WARNING "ZLIB not found; building without zlib support")
    endif()
else()
    message(STATUS "ZLIB support disabled")
endif()
