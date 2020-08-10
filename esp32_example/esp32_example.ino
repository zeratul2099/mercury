#include <WiFi.h>
#include <HTTPClient.h>
#include "time.h"
#include <ArduinoJson.h>

#include "DHTesp.h"

#define ENABLE_GxEPD2_GFX 0

#include <GxEPD2_BW.h>


#include <Fonts/FreeMonoBold12pt7b.h>
#include <Fonts/FreeMonoBold24pt7b.h>
#include <Fonts/FreeMonoBold9pt7b.h>
#define DHTTYPE DHT22

static const uint8_t EPD_BUSY = 4;  // to EPD BUSY
static const uint8_t EPD_CS   = 5;  // to EPD CS
static const uint8_t EPD_RST  = 21; // to EPD RST
static const uint8_t EPD_DC   = 22; // to EPD DC
static const uint8_t EPD_SCK  = 18; // to EPD CLK
static const uint8_t EPD_MISO = 19; // Master-In Slave-Out not used, as no data from display
static const uint8_t EPD_MOSI = 23; // to EPD DIN

GxEPD2_BW<GxEPD2_290, GxEPD2_290::HEIGHT> display(GxEPD2_290(/*CS=*/ EPD_CS, /*DC=*/ EPD_DC, /*RST=*/ EPD_RST, /*BUSY=*/ EPD_BUSY));

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
const char* ntpServer = "2.de.pool.ntp.org";
const long  gmtOffset_sec = 3600;
const int   daylightOffset_sec = 3600;


void IRAM_ATTR buttonPressed() {
  Serial.println("Interrupt");
  is_interrupt = 1;
}

void setup()
{
  // Initilize hardware:
  Serial.begin(115200);
  delay(100);
  display.init(115200); // uses standard SPI pins, e.g. SCK(18), MISO(19), MOSI(23), SS(5)
  SPI.end();
  SPI.begin(EPD_SCK, EPD_MISO, EPD_MOSI, EPD_CS);
  display.setRotation(1);
  display.setFont(&FreeMonoBold9pt7b);
  display.setTextColor(GxEPD_BLACK);
  
  pinMode(BUTTON_PIN, INPUT_PULLUP);
  attachInterrupt(digitalPinToInterrupt(BUTTON_PIN), buttonPressed, FALLING);

  sprintf(HOSTNAME, "esp32-s%d", SENSOR_ID);
  // Connect to the WiFi network (see function below loop)
  connectToWiFi(networkName, networkPswd);

  led(LOW); // LED off
  Serial.println(url);
  dht.setup(DHTPin, DHTesp::DHT22);
  configTime(gmtOffset_sec, daylightOffset_sec, ntpServer);
  
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
  struct tm timeinfo;
  getLocalTime(&timeinfo);
  char time_str[9];
  strftime(time_str, 9, "%H:%M:%S", &timeinfo);
  Serial.println(lastValues.temperature);
  Serial.println(lastValues.humidity);
  Serial.println(&timeinfo, "%H:%M:%S");
  showPartialUpdate(lastValues.temperature, lastValues.humidity, time_str, String(SENSOR_ID), 14);
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
  display.setFont(&FreeMonoBold9pt7b);
  display.setTextColor(GxEPD_BLACK);
  display.setFullWindow();
  display.firstPage();
  do
  {
    display.fillScreen(GxEPD_WHITE);
    display.setCursor(cursor_x, cursor_y+38);
    display.print("Connecting.");
  }
  while (display.nextPage());
  
  //display.updateWindow(cursor_x, cursor_y, box_w, 80, true);
}

void showConnectDot()
{
  do
  {
    display.print(".");
  }
  while (display.nextPage());
  //display.updateWindow(cursor_x, cursor_y, box_w, 80, true);
}

void showPartialUpdate(const float temperature, const float humidity, const String upper_right, const String id, const int text_w)
{
  String temperatureString = String(temperature,1);
  String humidityString = String(humidity,1);
  
  display.setFont(&FreeMonoBold24pt7b);
  display.setTextColor(GxEPD_BLACK);
  display.setFullWindow();
  display.firstPage();
  do
  {
    display.fillScreen(GxEPD_WHITE);
    //display.fillRect(box_x, box_y, box_w, box_h, GxEPD_WHITE);
    display.setCursor(cursor_x, cursor_y+38);
    display.print(temperatureString);
    display.print("C ");
    display.setCursor(cursor_x, cursor_y+90);
    display.print(humidityString);
    display.print("%");


    display.setFont(&FreeMonoBold12pt7b);
    display.setCursor(cursor_x + 280 - upper_right.length() * text_w, cursor_y + 4);
    display.print(upper_right);
    display.setCursor(cursor_x + 255, 124);
    display.print(id);
    }
  while (display.nextPage());
  
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
  if (httpCode == HTTP_CODE_OK || httpCode == HTTP_CODE_MOVED_PERMANENTLY) {
    String payload = http.getString();
    DynamicJsonDocument doc(8192);
    DeserializationError err = deserializeJson(doc, payload);
    if(err) {
        Serial.print(F("deserializeJson() failed with code "));
        Serial.println(err.c_str());
    } else {
    
      int arraySize =  doc.size();
 
      for (int i = 0; i< arraySize; i++){
         int sid = doc[i][0].as<long>();
        if (sid != SENSOR_ID) {
          String sname = doc[i][1].as<String>();
          float temp = doc[i][3].as<float>();
          float hum = doc[i][4].as<float>();
          Serial.println(sname);
          Serial.println(temp);
          Serial.println(hum);
          showPartialUpdate(temp, hum, sname, String(sid),  16);
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
