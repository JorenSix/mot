//Sends a midi event and reports how long it takes before
//it is returned
//
//Reporting is done via SERIAL
//
//This device must be programmed as SERIAL+MIDI
//
//The 'mot midi_roundtrip_test 0 0' can be used to echo messages
//

const int PERIOD_IN_MICROS = 200000;//200 ms
const int EVENT_DURATION_IN_MILLIS = 100;

volatile bool newEvent = false;
const int LED_PIN = LED_BUILTIN;

//timer for the measurements
IntervalTimer eventTimer;
elapsedMicros timeForResponse;

void setup() {
  pinMode(LED_PIN,OUTPUT);
  
  Serial.begin(115200);

  Serial.println("Starting timer");
  
	eventTimer.begin(setEventBoolean, PERIOD_IN_MICROS);
}

void setEventBoolean(){
  newEvent = true;
  
}

void loop(){
  if(newEvent){
    newEvent=false;
    
    timeForResponse = 0;
    usbMIDI.sendNoteOn(69, 120, 1);
    usbMIDI.send_now();
    
    digitalWrite(LED_PIN,!digitalRead(LED_PIN));
  }

  while (usbMIDI.read()) {
    Serial.print(timeForResponse);
    Serial.println(" microseconds for response");
  }
}
