#include <Arduino.h>

// --- Keypad map ---
char keys[4][3] = {
    {'1', '2', '3'}, {'4', '5', '6'}, {'7', '8', '9'}, {'*', '0', '#'}};

byte rowPins[4] = {4, 2, 3, 5};
byte colPins[3] = {6, 7, 8};

// --- Thời gian ---
const unsigned long holdThreshold = 300; // ms
const unsigned long debounceDelay = 40;  // ms
const unsigned long doubleThreshold =
    300; // ms, thời gian tối đa giữa 2 lần nhấn để coi là DOUBLE

// --- Debounce state ---
char candidateKey = 0;            // raw "ứng viên"
unsigned long candidateSince = 0; // thời điểm raw chuyển sang ứng viên
char debouncedKey = 0;            // trạng thái phím đã ổn định

// --- Event/Hold state ---
char activeKey = 0;             // phím đang được coi là "đang nhấn"
unsigned long keyDownStart = 0; // thời điểm bắt đầu nhấn
bool holdReported = false;      // đã in HOLD chưa

// --- Double state ---
char lastKey = 0; // phím lần trước
unsigned long lastReleaseTime = 0;
bool waitingSecondPress = false;

void setup() {
  Serial.begin(9600);

  for (int r = 0; r < 4; r++) {
    pinMode(rowPins[r], OUTPUT);
    digitalWrite(rowPins[r], HIGH); // idle HIGH
  }
  for (int c = 0; c < 3; c++) {
    pinMode(colPins[c], INPUT_PULLUP); // đọc LOW khi nhấn
  }

  Serial.println("[Arduino ready]");
}

char scanKeypadRaw() {
  for (int r = 0; r < 4; r++) {
    digitalWrite(rowPins[r], LOW); // kích hoạt hàng r
    delayMicroseconds(5);          // cho ổn định

    for (int c = 0; c < 3; c++) {
      if (digitalRead(colPins[c]) == LOW) {
        digitalWrite(rowPins[r], HIGH);
        return keys[r][c];
      }
    }
    digitalWrite(rowPins[r], HIGH);
  }
  return 0; // không nhấn
}

void loop() {
  unsigned long now = millis();

  // 1) Đọc raw
  char raw = scanKeypadRaw();

  // 2) Debounce
  if (raw != candidateKey) {
    candidateKey = raw;
    candidateSince = now;
  }

  if ((now - candidateSince >= debounceDelay) &&
      (debouncedKey != candidateKey)) {
    debouncedKey = candidateKey;

    // 3) Phát sự kiện theo cạnh
    if (debouncedKey != 0) {
      // 0 -> key : bắt đầu nhấn
      activeKey = debouncedKey;
      keyDownStart = now;
      holdReported = false;
      // Không in PRESS ngay, chờ nhả hoặc HOLD
    } else {
      // key -> 0 : nhả ra
      if (activeKey != 0 && !holdReported) {
        if (waitingSecondPress && activeKey == lastKey &&
            (now - lastReleaseTime <= doubleThreshold)) {
          Serial.print("DOUBLE:");
          Serial.println(activeKey);
          waitingSecondPress = false; // reset
        } else {
          // chưa biết DOUBLE hay không, bắt đầu chờ
          lastKey = activeKey;
          lastReleaseTime = now;
          waitingSecondPress = true;
        }
      }
      activeKey = 0;
    }
  }

  // 4) Kiểm tra HOLD
  if (activeKey != 0 && !holdReported &&
      (now - keyDownStart >= holdThreshold)) {
    Serial.print("HOLD:");
    Serial.println(activeKey);
    holdReported = true;
  }

  // 5) Timeout DOUBLE -> coi như PRESS
  if (waitingSecondPress && (now - lastReleaseTime > doubleThreshold)) {
    Serial.print("PRESS:");
    Serial.println(lastKey);
    waitingSecondPress = false;
  }
}
