## Summary

Quick and dirty tool to create ADIF logs from WSPRnet spot database dumps.

## Usage

This tool reads a [WSPRnet spot database dump](https://wsprnet.org/drupal/downloads) in CSV format
from standard input, filters the spots according to the reporter call sign provided as an argument,
and writes an [ADIF](https://adif.org/) log to standard output.

It generates one record per transmitter call sign, choosing the spot with the best SpotQ metric.

### Example

```
gunzip -c wsprspots-2021-01.csv.gz | wsprspots DO5EU > wsprspots-2021-01.adi
```

**Generated ADIF log:**


```
WSPR spots for DO5EU
<ADIF_VER:5>3.1.1<CREATED_TIMESTAMP:15>20210212 231649<PROGRAMID:9>wsprspots<PROGRAMVERSION:5>0.0.5<EOH>
<APP_WSPRSPOTS_SPOTID:10:N>2730817432<QSO_DATE:8>20210110<TIME_ON:4>2338<OPERATOR:5>DO5EU<MY_GRIDSQUARE:6>JO62qm<RST_SENT:6>-15 dB<RST_RCVD:0><APP_WSPRSPOTS_SNR:3:N>-15<APP_WSPRSPOTS_DRIFT:1:N>0<FREQ:8>1.014012<CALL:6>DP0GVN<GRIDSQUARE:6>IB59ui<RX_PWR:6>0.5012<DISTANCE:5>13805<BAND:3>30m<MODE:4>WSPR<QSO_RANDOM:1>Y<SWL:1>Y<APP_WSPRSPOTS_SPOTQ:5:N>16068<QSLMSG:84>WSPR spot on 30 m with 500 mW (27 dBm), SNR -15 dB, drift +0 Hz/s, distance 13805 km<COMMENT:26>WSPRnet spot ID 2730817432<NOTES:11>SpotQ 16068<EOR>
```

## Implementation notes

The SpotQ metric is calculated according to the formula by Phil VK7JJ Perite (cf.
<http://wsprd.vk7jj.com/> → *FAQ* → *SpotQ*), but the results may differ slightly from the reference
implementation due to differences in rounding.

The code might benefit from restructuring.
