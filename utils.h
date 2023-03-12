#pragma once

#include <string>
#include <locale>
#include <cstdio>
#include <iostream>
#include <algorithm>

inline std::string codepoint_to_utf8(char32_t cp)
{
    char buff[16];
    snprintf(buff, sizeof(buff), "%s", (char *) & cp);
    std::string buffAsStdStr = buff;

    return buffAsStdStr;
}

template<size_t N>
struct StringLiteral {
    constexpr StringLiteral(const char(&str)[N]) {
        std::copy_n(str, N, value);
    }

    char value[N];
};
