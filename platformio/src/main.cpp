#include <Arduino.h>

char keys[4][3] = {
    {'1', '2', '3'}, {'4', '5', '6'}, {'7', '8', '9'}, {'*', '0', '#'}};

byte rowPins[4] = {4, 2, 3, 5};
byte colPins[3] = {6, 7, 8};

const unsigned long holdThreshold = 300; // ms
const unsigned long debounceDelay = 15;  // ms

// --- Debounce state ---
char candidateKey = 0;            // giá trị raw mới nhất làm "ứng viên"
unsigned long candidateSince = 0; // thời điểm raw chuyển sang "ứng viên"
char debouncedKey = 0;            // trạng thái phím đã qua debounce (ổn định)

// --- Event/hold state ---
char activeKey = 0; // phím đang được coi là "đang nhấn" để theo dõi hold
unsigned long keyDownStart = 0; // thời điểm bắt đầu nhấn (đã debounce)
bool holdReported = false;      // đã in HOLD cho lần nhấn hiện tại chưa

void setup() {
  Serial.begin(9600);

  for (int r = 0; r < 4; r++) {
    pinMode(rowPins[r], OUTPUT);
    digitalWrite(rowPins[r], HIGH); // idle: HIGH
  }
  for (int c = 0; c < 3; c++) {
    pinMode(colPins[c], INPUT_PULLUP); // đọc LOW khi nhấn
  }

  Serial.println("[Arduino ready]");
}

char scanKeypadRaw() {
  for (int r = 0; r < 4; r++) {
    digitalWrite(rowPins[r], LOW); // kích hoạt hàng r
    delayMicroseconds(5);          // cho đường tín hiệu ổn định
    for (int c = 0; c < 3; c++) {
      if (digitalRead(colPins[c]) == LOW) { // nhấn => LOW
        digitalWrite(rowPins[r], HIGH);     // khôi phục hàng r
        return keys[r][c];
      }
    }
    digitalWrite(rowPins[r], HIGH); // khôi phục trước khi chuyển hàng khác
  }
  return 0; // không nhấn phím nào
}

void loop() {
  unsigned long now = millis();

  // 1) Đọc raw
  char raw = scanKeypadRaw();

  // 2) Debounce theo thời gian
  if (raw != candidateKey) {
    candidateKey = raw;
    candidateSince = now;
  }
  // Khi ứng viên giữ nguyên đủ lâu, cập nhật debouncedKey
  if ((now - candidateSince >= debounceDelay) &&
      (debouncedKey != candidateKey)) {
    debouncedKey = candidateKey;

    // 3) Phát sự kiện theo cạnh (sau khi đã debounce)
    if (debouncedKey != 0) {
      // 0 -> key : bắt đầu nhấn
      activeKey = debouncedKey;
      keyDownStart = now;
      holdReported = false;
      // Không in PRESS ngay tại đây; đợi đến khi nhả hoặc HOLD đủ lâu
    } else {
      // key -> 0 : nhả ra
      if (activeKey != 0 && !holdReported) {
        Serial.print("PRESS:");
        Serial.println(activeKey);
      }
      activeKey = 0;
    }
  }

  // 4) Kiểm tra HOLD khi đang giữ
  if (activeKey != 0 && !holdReported &&
      (now - keyDownStart >= holdThreshold)) {
    Serial.print("HOLD:");
    Serial.println(activeKey);
    holdReported = true;
  }
}
