#pragma once

#include <string>
#include <locale>
#include <cstdio>

inline std::string codepoint_to_utf8(char32_t cp)
{
    char buff[16];
    snprintf(buff, sizeof(buff), "%c", cp);
    std::string buffAsStdStr = buff;

    return buffAsStdStr;
}

struct RGB
{
    float r, g, b;

    explicit RGB(float r, float g, float b)
        : r(r), g(g), b(b) {}

    RGB()
        : r(0.0f), g(0.0f), b(0.0f) {}

    static RGB random()
    {
        return RGB{ (float)(rand() % 255), (float)(rand() % 255), (float)(rand() % 255) };
    }

    operator TCOD_ColorRGB()
    {
        return { (uint8_t)r, (uint8_t)g, (uint8_t)b };
    }
};

inline RGB operator""_rgb(const char* hexValue, size_t size)
{
    assert(size == 7);
    int r, g, b;
    sscanf_s(hexValue, "#%02x%02x%02x", &r, &g, &b);
    return RGB{
        (r & 0xFF) / 1.0f,
        (g & 0xFF) / 1.0f,
        (b & 0xFF) / 1.0f
    };
}


struct HSL
{
    float h, s, l;

    explicit HSL(float h, float s, float l)
        : h(h), s(s), l(l)
    {}

    HSL()
        : h(0.0f), s(0.0f), l(0.0f)
    {}

    operator TCOD_ColorRGB()
    {
        return TCOD_color_HSV(h, s, l);
    }

    operator RGB()
    {
        auto c = TCOD_color_HSV(h, s, l);
        return RGB{ (float)c.r, (float)c.g, (float)c.b };
    }
};