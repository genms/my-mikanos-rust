#include <new>

#include "usb/memory.hpp"
#include "usb/device.hpp"
#include "usb/classdriver/mouse.hpp"
#include "usb/xhci/xhci.hpp"
#include "usb/xhci/trb.hpp"
#include "error.hpp"
#include "logger.hpp"

char xhc_buf[sizeof(usb::xhci::Controller)];
usb::xhci::Controller* xhc;
typedef int XHC_HANDLE;

extern "C" XHC_HANDLE UsbInitXhc(uint64_t xhc_mmio_base) {
  xhc = new(xhc_buf) usb::xhci::Controller(xhc_mmio_base);

  auto err = xhc->Initialize();
  Log(kDebug, "xhc.Initialize: %s\n", err.Name());

  Log(kInfo, "xHC starting\n");
  xhc->Run();

  return 0;
}

typedef void (*MouseObserverType)(int8_t, int8_t);

extern "C" void UsbConfigurePort(XHC_HANDLE xhc_handle, MouseObserverType mouse_observer) {
  usb::HIDMouseDriver::default_observer = mouse_observer;

  for (int i = 1; i <= xhc->MaxPorts(); ++i) {
    auto port = xhc->PortAt(i);
    Log(kDebug, "Port %d: IsConnected=%d\n", i, port.IsConnected());

    if (port.IsConnected()) {
      if (auto err = ConfigurePort(*xhc, port)) {
        Log(kError, "failed to configure port: %s at %s:%d\n",
            err.Name(), err.File(), err.Line());
        continue;
      }
    }
  }
}

extern "C" void UsbReceiveEvent(XHC_HANDLE xhc_handle) {
  if (auto err = ProcessEvent(*xhc)) {
    Log(kError, "Error while ProcessEvent: %s at %s:%d\n",
        err.Name(), err.File(), err.Line());
  }
}

extern "C" void __cxa_pure_virtual() {
  while (1) __asm__("hlt");
}
