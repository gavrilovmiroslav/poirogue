#pragma once

// map width/height
#define WIDTH 80
#define HEIGHT 52

// room config
#define ROOM_COUNT 20
#define MIN_TILES_PER_ROOM 10

// people and regions
#define REGION_COUNT 6
#define PEOPLE_COUNT 10

// actions
#define ACTION_POINTS_PER_TURN 16
#define ACTION_POINTS_PLAYER_BONUS 0

#define ATTRIBUTE_SPEED_NORM 100
#define ATTRIBUTE_SIGHT_NORM 20

struct Colors
{
	float visible_hue = 255.0f;
	float visible_sat = 0.5f;
	float visible_shift_mid = 0.75f;
	float visible_shift_far = 0.45f;
	float visible_shift_very_far = 0.25f;

	float memory_hue = 240.0f;
	float memory_sat = 0.1f;
	float memory_lit = 0.5f;

	float shimmer_hue = 180.0f;
	float shimmer_stripe_strength = 20.0f;
	float shimmer_stripe_speed = 0.05f;
	float shimmer_stripe_width = 0.25f;
};