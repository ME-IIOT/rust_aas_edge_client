// Author: Pham-Minh-Khai Hoang (khai.hoang@yacoub.de), Manh-Linh Phan (manh.linh.phan@yacoub.de), Xuan-Thuy Dang (xuan.thuy.dang@yacoub.de), Markus Rentschler
const reportWebVitals = onPerfEntry => {
  if (onPerfEntry && onPerfEntry instanceof Function) {
    import('web-vitals').then(({ getCLS, getFID, getFCP, getLCP, getTTFB }) => {
      getCLS(onPerfEntry);
      getFID(onPerfEntry);
      getFCP(onPerfEntry);
      getLCP(onPerfEntry);
      getTTFB(onPerfEntry);
    });
  }
};

export default reportWebVitals;
