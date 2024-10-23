#define LED 2
#include <BLEMidi.h>

hw_timer_t *timer = NULL;

boolean sendMidi = false;

unsigned long midiMessagSendTime = 0;

void ARDUINO_ISR_ATTR onTimer(){
  // Increment the counter and set the time of ISR
  sendMidi = true;
  digitalWrite(LED, !digitalRead(LED));
}

void onNoteOn(uint8_t channel, uint8_t note, uint8_t velocity, uint16_t timestamp){
  unsigned long midiMessageReceiveTime = micros();
  unsigned long roundtripMidiMessageTime  =  midiMessageReceiveTime - midiMessagSendTime;
  Serial.printf("Received note on : round trip time %d microsecond\n", roundtripMidiMessageTime);
}

void setup() {
  pinMode(LED, OUTPUT);

  //set timer callback function
  timer = timerBegin(1000000);
  // Attach onTimer function to our timer.
  timerAttachInterrupt(timer, &onTimer);
  // Set alarm to call onTimer function every second (value in microseconds).
  // Repeat the alarm (third parameter) with unlimited count = 0 (fourth parameter).
  timerAlarm(timer, 1000000, true, 0);

  Serial.begin(115200);
  Serial.println("Initializing bluetooth");

  BLEMidiServer.begin("MIDI_Roundtrip"); 
  BLEMidiServer.setNoteOnCallback(onNoteOn);
  while( ! BLEMidiServer.isConnected() ){
    Serial.println("Waiting for BLE MIDI connection...");
    delay(1000);
  }
}

void loop() {
  if(sendMidi){
    sendMidi = false;
    midiMessagSendTime = micros();
    BLEMidiServer.noteOn(0, 69, 127);
    Serial.println("Midi Message send");
  }
}