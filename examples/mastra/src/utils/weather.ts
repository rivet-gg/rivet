import type { WeatherData } from '../types/index.js';

const WEATHER_CONDITIONS: Record<number, string> = {
  0: "Clear sky",
  1: "Mainly clear",
  2: "Partly cloudy",
  3: "Overcast",
  45: "Foggy",
  51: "Light drizzle",
  61: "Slight rain",
  63: "Moderate rain",
  65: "Heavy rain",
  71: "Slight snow",
  73: "Moderate snow",
  95: "Thunderstorm"
};

export async function getWeatherData(location: string): Promise<WeatherData> {
  try {
    // Geocoding API call
    const geocodingUrl = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(location)}&count=1`;
    const geocodingResponse = await fetch(geocodingUrl, {
      signal: AbortSignal.timeout(5000)
    });
    const geocodingData = await geocodingResponse.json();

    if (!geocodingData.results?.[0]) {
      throw new Error(`Location '${location}' not found`);
    }

    const { latitude, longitude, name } = geocodingData.results[0];

    // Weather API call  
    const weatherUrl = `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,wind_gusts_10m,weather_code`;
    const weatherResponse = await fetch(weatherUrl, {
      signal: AbortSignal.timeout(5000)
    });
    const weatherData = await weatherResponse.json();

    return {
      temperature: weatherData.current.temperature_2m,
      feelsLike: weatherData.current.apparent_temperature,
      humidity: weatherData.current.relative_humidity_2m,
      windSpeed: weatherData.current.wind_speed_10m,
      windGust: weatherData.current.wind_gusts_10m,
      conditions: WEATHER_CONDITIONS[weatherData.current.weather_code] || "Unknown",
      location: name,
      timestamp: new Date().toISOString()
    };
  } catch (error) {
    console.error('Weather API error:', error);
    throw error;
  }
}