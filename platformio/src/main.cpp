#include <Arduino.h>

char keys[4][3] = {
    {'1', '2', '3'}, {'4', '5', '6'}, {'7', '8', '9'}, {'*', '0', '#'}};

byte rowPins[4] = {4, 2, 3, 5};
byte colPins[3] = {6, 7, 8};

char lastKeyState = 0;
unsigned long pressStartTime = 0;
bool reported = false; // đã in ra sự kiện cho lần nhấn này chưa

const unsigned long holdThreshold = 300; // ms
const unsigned long debounceDelay = 40;  // ms

void setup() {
  Serial.begin(9600);
  for (int r = 0; r < 4; r++) {
    pinMode(rowPins[r], OUTPUT);
    digitalWrite(rowPins[r], HIGH);
  }
  for (int c = 0; c < 3; c++) {
    pinMode(colPins[c], INPUT_PULLUP);
  }
  Serial.println("[Arduino ready]");
}

char scanKeypad() {
  for (int r = 0; r < 4; r++) {
    digitalWrite(rowPins[r], LOW);
    for (int c = 0; c < 3; c++) {
      if (digitalRead(colPins[c]) == LOW) {
        digitalWrite(rowPins[r], HIGH);
        return keys[r][c];
      }
    }
    digitalWrite(rowPins[r], HIGH);
  }
  return 0;
}

void loop() {
  char key = scanKeypad();
  unsigned long now = millis();

  if (key != 0) {
    if (key != lastKeyState) {
      // bắt đầu nhấn phím mới
      static unsigned long lastDebounce = 0;
      if (now - lastDebounce > debounceDelay) {
        pressStartTime = now;
        lastKeyState = key;
        reported = false;
        lastDebounce = now;
      }
    } else {
      // phím đang được giữ
      if (!reported && (now - pressStartTime > holdThreshold)) {
        Serial.print("HOLD:");
        Serial.println(key);
        reported = true;
      }
    }
  } else {
    // phím được nhả ra
    if (lastKeyState != 0 && !reported) {
      // nếu chưa từng báo HOLD, thì báo PRESS
      Serial.print("PRESS:");
      Serial.println(lastKeyState);
    }
    lastKeyState = 0;
  }
}
