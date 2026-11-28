"""Narrow BLE probe for Vial 0.7.5 plus portable diagnostic logging."""
import json
import logging
import os
import platform
import sys
from logging.handlers import RotatingFileHandler

UPSTREAM = "e42622c352b69eff523a467718db487ab9c9cbf2"
LOG_NAME = "vial-ble-diagnostic.log"
PROBE_VID = 0x4C4B
PROBE_PID = 0x4643
PROBE_USAGE_PAGE = 0xFF60
PROBE_USAGE = 0x61
PROBE_SERIAL = "vial:f64c2b3c:ble-probe"


def enable_portable_log():
    if getattr(sys, "frozen", False):
        directory = os.path.dirname(sys.executable)
    else:
        directory = os.path.dirname(os.path.abspath(__file__))
    try:
        handler = RotatingFileHandler(
            os.path.join(directory, LOG_NAME), maxBytes=5 * 1024 * 1024,
            backupCount=1, encoding="utf-8")
        handler.setFormatter(logging.Formatter(
            "%(asctime)s - %(levelname)s - %(message)s"))
        logging.getLogger().addHandler(handler)
    except OSError:
        # The isolated application-data log remains available if the extracted
        # directory is not writable. Never modify an installed Vial folder.
        logging.exception("BLE_DIAG portable log unavailable")
    logging.info("BLE_DIAG_START upstream=%s python=%s windows=%s",
                 UPSTREAM, platform.python_version(), platform.platform())


def enumerate_for_diagnostic(hid, quiet, via_stack_json, sideload_vid, sideload_pid):
    original_devices = hid.enumerate()
    devices = []
    if not quiet:
        logging.info("BLE_DIAG_REFRESH total_hid=%d", len(original_devices))
    for device in original_devices:
        vendor = device.get("vendor_id")
        product = device.get("product_id")
        raw_hid = device.get("usage_page") == 0xff60 and device.get("usage") == 0x61
        serial = device.get("serial_number")
        probe_candidate = (
            (vendor, product) == (PROBE_VID, PROBE_PID)
            and raw_hid
            and not serial
        )
        if probe_candidate:
            candidate = dict(device)
            candidate["serial_number"] = PROBE_SERIAL
            devices.append(candidate)
            if not quiet:
                logging.warning(
                    "BLE_DIAG_PROBE_ADMIT vid=%04X pid=%04X path=%r synthetic_serial=%s",
                    vendor, product, device.get("path"), PROBE_SERIAL)
        else:
            devices.append(device)
        if quiet:
            continue
        # Record this firmware's interfaces and Vial-style raw-HID interfaces.
        # Do not collect unrelated keyboard, macro, account or layout data.
        if not ((vendor, product) == (0x4c4b, 0x4643) or raw_hid):
            continue
        path = device.get("path", b"")
        if isinstance(path, bytes):
            path = path.decode("utf-8", errors="replace")
        record = {
            "vendor_id": vendor, "product_id": product,
            "serial_number": serial,
            "manufacturer_string": device.get("manufacturer_string"),
            "product_string": device.get("product_string"),
            "usage_page": device.get("usage_page"), "usage": device.get("usage"),
            "interface_number": device.get("interface_number"), "path": path,
            "raw_hid": raw_hid,
            "vial_serial_magic": isinstance(serial, str) and "vial:f64c2b3c" in serial,
            "probe_candidate": probe_candidate,
            "sideload_match": (vendor, product) == (sideload_vid, sideload_pid),
            "via_definition_match": str(vendor * 65536 + product) in via_stack_json["definitions"],
        }
        logging.info("BLE_DIAG_DEVICE %s", json.dumps(record, ensure_ascii=True, sort_keys=True))
    return devices
