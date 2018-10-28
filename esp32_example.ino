#include <WiFi.h>
#include <HTTPClient.h>
#include <NTPClient.h>
#include <WiFiUdp.h>
#include <ArduinoJson.h>

#include "DHTesp.h"

#include <GxEPD.h>
#include <GxGDEH029A1/GxGDEH029A1.cpp>
#include <GxIO/GxIO_SPI/GxIO_SPI.cpp>
#include <GxIO/GxIO.cpp>
#include <Fonts/FreeSansBold12pt7b.h>
#include <Fonts/FreeSansBold24pt7b.h>

#define DHTTYPE DHT22

// WiFi network name and password:
const char * networkName = ""; // fill out
const char * networkPswd = ""; // fill out

const char * url = "http://<mercury>:5001/api/send"; // change to hostname of mercury instance
const char * rcUrl = "http://<mercury>:5001/api/latest"; // change to hostname of mercury instance

const int BUTTON_PIN = 26;
const int LED_PIN = 5;
const int LED_ON_SEND = 1;
const int SENSOR_ID = ; // adjust, every sensor needs another id
char HOSTNAME[10];

const int SEND_DELAY = 60; // update every n seconds
// Display
GxIO_Class io(SPI, SS, 22, 21);
GxEPD_Class display(io, 16, 4);


uint16_t box_x = 0;
uint16_t box_y = 0;
uint16_t box_w = 296;
uint16_t box_h = 124;
uint16_t cursor_x = box_x + 16;
uint16_t cursor_y = box_y + 16;
int counter = SEND_DELAY;
int is_interrupt = 0;

// DHT Sensor
const int DHTPin = 16;
// Initialize DHT sensor.
DHTesp dht;

// ntp
WiFiUDP ntpUDP;
NTPClient timeClient(ntpUDP, "2.de.pool.ntp.org", 3600, 60000); // adjust to your timezone, TODO: find better way



void IRAM_ATTR buttonPressed() {
  Serial.println("Interrupt");
  is_interrupt = 1;
}

void setup()
{
  // Initilize hardware:
  Serial.begin(115200);
  
  display.init();
  // clean
  display.fillRect(0, 0, 296, 128, GxEPD_WHITE);
  display.update();
  
  pinMode(BUTTON_PIN, INPUT_PULLUP);
  attachInterrupt(digitalPinToInterrupt(BUTTON_PIN), buttonPressed, FALLING);

  sprintf(HOSTNAME, "esp32-s%d", SENSOR_ID);
  // Connect to the WiFi network (see function below loop)
  connectToWiFi(networkName, networkPswd);

  led(LOW); // LED off
  Serial.println(url);
  dht.setup(DHTPin, DHTesp::DHT22);
  timeClient.begin();
}

void loop()
{
  if (WiFi.status() != WL_CONNECTED)
  {
    connectToWiFi(networkName, networkPswd);
  }
  
  if (is_interrupt == 1) {
    is_interrupt = 0;
    showOtherValues();
    TempAndHumidity lastValues = dht.getTempAndHumidity();
    showCurrentValues(lastValues);
  }
  if (counter >= SEND_DELAY) {
    counter = 0;
    TempAndHumidity lastValues = dht.getTempAndHumidity();
    showCurrentValues(lastValues);
    char temp[6];
    char hum[6];
    ftoa(temp,lastValues.temperature);
    ftoa(hum,lastValues.humidity);
    if (LED_ON_SEND == 1)
    {
      led(LOW); // Turn on LED  
    }
    
    sendValues(url, SENSOR_ID, temp, hum);

    if (LED_ON_SEND == 1)
    {
      led(LOW); // Turn off LED
    }
  }
  counter += 1;
  delay(1000);
}

void showCurrentValues(TempAndHumidity lastValues) {
  timeClient.update();
  Serial.println(lastValues.temperature);
  Serial.println(lastValues.humidity);
  Serial.println(timeClient.getFormattedTime());
  showPartialUpdate(lastValues.temperature, lastValues.humidity, timeClient.getFormattedTime(), String(SENSOR_ID), 12);
}

void led(int val)
{
  //Serial.println(val);
  digitalWrite(LED_PIN, val);
}

void connectToWiFi(const char * ssid, const char * pwd)
{
  int ledState = 0;

  Serial.println("Connecting to WiFi network: " + String(ssid));

  WiFi.begin(ssid, pwd);
  tcpip_adapter_set_hostname(TCPIP_ADAPTER_IF_STA, HOSTNAME);
  showConnecting();
  while (WiFi.status() != WL_CONNECTED) 
  {
    // Blink LED while we're connecting:
    led(ledState);
    ledState = (ledState + 1) % 2; // Flip ledState
    delay(500);
    showConnectDot();
    Serial.print(".");
  }
  led(LOW);
  Serial.println();
  Serial.println("WiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
}

void showConnecting()
{
  const char* name = "FreeSansBold12pt7b";
  const GFXfont* f = &FreeSansBold12pt7b;
  display.setFont(f);
  display.setRotation(45);
  display.setTextColor(GxEPD_BLACK);
  display.setCursor(cursor_x, cursor_y+38);
  display.print("Connecting.");
  display.updateWindow(cursor_x, cursor_y, box_w, 80, true);
}

void showConnectDot()
{
  display.print(".");
  display.updateWindow(cursor_x, cursor_y, box_w, 80, true);
}

void showPartialUpdate(const float temperature, const float humidity, const String upper_right, const String id, const int text_w)
{
  String temperatureString = String(temperature,1);
  String humidityString = String(humidity,1);
  const char* name = "FreeSansBold24pt7b";
  const GFXfont* f = &FreeSansBold24pt7b;
  
  display.setRotation(45);
  display.setFont(f);
  display.setTextColor(GxEPD_BLACK);

  display.fillRect(box_x, box_y, box_w, box_h, GxEPD_WHITE);
  display.setCursor(cursor_x, cursor_y+38);
  display.print(temperatureString);
  display.print("C ");
  display.setCursor(cursor_x, cursor_y+90);
  display.print(humidityString);
  display.print("%");

  name = "FreeSansBold12pt7b";
  f = &FreeSansBold12pt7b;
  display.setFont(f);
  display.setCursor(cursor_x + 280 - upper_right.length() * text_w, cursor_y + 4);
  display.print(upper_right);
  display.setCursor(cursor_x + 255, 124);
  display.print(id);
  display.updateWindow(box_x, box_y, box_w, box_h, true);
}

void sendValues(const char * url, const int id, const char * temp, const char * hum)
{
  HTTPClient http;
  char message[64];
  sprintf(message, "%s?id=%d&t=%s&h=%s", url, id, temp, hum);
  http.begin(message);
  int httpCode = http.GET();   //Make the request

    if (httpCode > 0) { //Check for the returning code

        String payload = http.getString();
        Serial.println(httpCode);
        Serial.println(payload);
      }

    else {
      Serial.println("Error on HTTP request");
      Serial.println(httpCode);
    }

    http.end(); //Free the resources
  }

void showOtherValues() {
  HTTPClient http;
  http.begin(rcUrl);
  int httpCode = http.GET();
  int timeout_counter = 0;
  if (httpCode > 0) {
    String jsonMessage = http.getString();
    char charBuf[3000];
    jsonMessage.toCharArray(charBuf, 3000);
    StaticJsonBuffer<3000> JSONBuffer;
    JsonArray& parsed = JSONBuffer.parseArray(charBuf);
    if (parsed.success()) {
      int arraySize =  parsed.size();
 
      for (int i = 0; i< arraySize; i++){
 
        JsonArray& sensorData = parsed[i];
        int sid = sensorData[0];
        if (sid != SENSOR_ID) {
          String sname = sensorData[1];
          float temp = sensorData[3];
          float hum = sensorData[4];
          Serial.println(sname);
          Serial.println(temp);
          Serial.println(hum);
          showPartialUpdate(temp, hum, sname, String(sid), 16);
          // prevent button bouncing
          is_interrupt = 0;
          while (is_interrupt == 0) {
            if (timeout_counter >= 6) {
              break;
            }
            timeout_counter++;
            delay(500);
          }
          timeout_counter = 0;
          is_interrupt = 0;
        }
      } 

    }else {
        Serial.println("Parsing failed");
    }
  }
  // prevent button bouncing
  is_interrupt = 0;
  http.end();
}

int ftoa(char *a, float f)  //translates floating point readings into strings to send over the air
{
  int left=int(f);
  float decimal = f-left;
  int right = decimal *100; //2 decimal points
  if (right > 9) {  //if the decimal has two places already. Otherwise
    sprintf(a, "%d.%d",left,right);
  } else { 
    sprintf(a, "%d.0%d",left,right); //pad with a leading 0
  }
}
