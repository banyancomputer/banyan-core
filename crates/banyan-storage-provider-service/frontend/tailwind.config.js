/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        mainBackground: "var(--main-background)",
        secondaryBackground: "var(--secondary-background)",
        lightText: "var(--light-text)",
        darkText: "var(--dark-text)",
        tableHead: "var(--table-head)",
        tableBody: "var(--table-body)",
        tableExtend: "var(--table-extend)",
        tableExtendText: "var(--table-extend-text)",
        storageBackground: "var(--storage-background)",
        highlightColor: "var(--highlight-color)",
        redHighlightColor: "var(--red-highlight-color)",
        chartLight: "var(--chart-light)",
        chartDark: "var(--chart-dark)",
        tableBorder: "var(--table-border)",
        contextMenuBackground: "var(--context-menu-background)",
        contextMenuHoverackground: "var(--context-menu-hover-background)",
      },
      maxHeight: {
        table: "1000px",
        notifications: "400px",
      },
      maxWidth: {
        wrapper: "1440px",
      },
      borderWidth: {
        1: "1px",
      },
      padding: {
        2.5: "10px",
        1.5: "6px",
      },
      fontFamily: {
        inter: ["Inter"],
        boogy: ["BoogyBrut"],
      },
      fontSize: {
        10: ["10px", { lineHeight: "18px" }],
        12: ["12px", { lineHeight: "18px" }],
        14: ["14px", { lineHeight: "20px" }],
        16: ["16px", { lineHeight: "18px" }],
        18: ["18px", { lineHeight: "18px" }],
        20: ["20px", { lineHeight: "32px" }],
        28: ["28px", { lineHeight: "32px" }],
        40: ["40px", { lineHeight: "50px" }],
        42: ["42px", { lineHeight: "50px" }],
        64: ["64px", { lineHeight: "64px" }],
      },
    },
  },
  plugins: [],
};
