## Synopsis

This tool reads a [WSPRnet spot database dump](https://wsprnet.org/drupal/downloads) in CSV format
from standard input, identifies QSOs by correlating mutual WSPR spots within a four‐minute time
window and writes an [ADIF](https://adif.org/) log to standard output.

## Usage

### Example

```
gunzip -c wsprspots-2021-01.csv.gz | wsprspots DO5EU > wsprspots-2021-01.adi
```

**Generated ADIF log:**


```
Mutual WSPR spots for DO5EU
<ADIF_VER:5>3.1.1<CREATED_TIMESTAMP:15>20210219 204507<PROGRAMID:9>wsprspots<PROGRAMVERSION:5>0.1.0<EOH>
<QSO_DATE:8>20210112<TIME_ON:4>2120<QSO_DATE_OFF:8>20210112<TIME_OFF:4>2124<OPERATOR:5>DO5EU<CALL:6>DP0GVN<MY_GRIDSQUARE:6>JO62qm<GRIDSQUARE:6>IB59ui<RST_RCVD:3>-29<RST_SENT:3>-29<FREQ:8>3.570003<RX_FREQ:8>7.040022<BAND:3>80m<BAND_RX:3>40m<TX_PWR:6>5.0119<RX_PWR:6>0.5012<DISTANCE:5>13805<QSLMSG:100>2-way WSPR spot on 80 m (RX 40 m) with 500 mW (27 dBm), SNR -29 dB, drift +0 Hz/s, distance 13805 km<COMMENT:100>2-way WSPR spot on 80 m (RX 40 m) with 500 mW (27 dBm), SNR -29 dB, drift +0 Hz/s, distance 13805 km<NOTES:39>WSPRnet spot IDs 2736249418, 2736254754<MODE:4>WSPR<QSO_RANDOM:1>Y<EOR>
```

## Implementation notes

There is a lot of potential for optimisation in this code.
