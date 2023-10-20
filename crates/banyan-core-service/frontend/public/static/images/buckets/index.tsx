import { SVGProps } from "react";

export const Upload = () => <svg width="47" height="46" viewBox="0 0 47 46" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="3.5" y="3" width="40" height="40" rx="20" fill="#EFF1F5" />
    <path d="M20.168 26.3333L23.5013 23M23.5013 23L26.8346 26.3333M23.5013 23V30.5M30.168 26.9524C31.1859 26.1117 31.8346 24.8399 31.8346 23.4167C31.8346 20.8854 29.7826 18.8333 27.2513 18.8333C27.0692 18.8333 26.8989 18.7383 26.8064 18.5814C25.7197 16.7374 23.7133 15.5 21.418 15.5C17.9662 15.5 15.168 18.2982 15.168 21.75C15.168 23.4718 15.8642 25.0309 16.9904 26.1613" stroke="#4A5578" strokeWidth="1.66667" strokeLinecap="round" strokeLinejoin="round" />
    <rect x="3.5" y="3" width="40" height="40" rx="20" stroke="white" strokeWidth="6" />
</svg>;

export const BucketIcon = (params: SVGProps<any>) => <svg width="120" height="89" viewBox="0 0 120 89" fill="none" xmlns="http://www.w3.org/2000/svg" {...params}>
    <path d="M5.26172 8H114.404V65C114.404 78.2548 103.659 89 90.4039 89H29.2617C16.0069 89 5.26172 78.2548 5.26172 65V8Z" fill="url(#paint0_linear_2680_2634)" />
    <rect x="0.162109" y="3" width="119.342" height="39" rx="10" fill="#41767D" />
    <rect x="0.162109" width="119.342" height="39" rx="10" fill="#8BA7AC" />
    <rect x="5.26172" y="2" width="109.142" height="32" rx="7" fill="url(#paint1_linear_2680_2634)" />
    <rect x="38.9238" y="49" width="43.8609" height="9" rx="4.5" fill="#99CBD5" />
    <rect x="38.9238" y="49" width="43.8609" height="7" rx="3.5" fill="#29717B" />
    <defs>
        <linearGradient id="paint0_linear_2680_2634" x1="59.8328" y1="8" x2="59.8328" y2="89" gradientUnits="userSpaceOnUse">
            <stop stopColor="#B2EDF5" />
            <stop offset="0.291667" stopColor="#B3C2C6" />
            <stop offset="1" stopColor="#346D75" />
        </linearGradient>
        <linearGradient id="paint1_linear_2680_2634" x1="59.8328" y1="2" x2="59.8328" y2="34" gradientUnits="userSpaceOnUse">
            <stop stopColor="#AEC3C6" />
            <stop offset="1" stopColor="#3A7077" />
        </linearGradient>
    </defs>
</svg>;

export const Folder = (params: SVGProps<any>) => <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg" {...params}>
    <path d="M10.8337 5.83333L9.90404 3.9741C9.63649 3.439 9.50271 3.17144 9.30313 2.97597C9.12664 2.80311 8.91393 2.67164 8.68039 2.59109C8.4163 2.5 8.11716 2.5 7.5189 2.5H4.33366C3.40024 2.5 2.93353 2.5 2.57701 2.68166C2.2634 2.84144 2.00844 3.09641 1.84865 3.41002C1.66699 3.76654 1.66699 4.23325 1.66699 5.16667V5.83333M1.66699 5.83333H14.3337C15.7338 5.83333 16.4339 5.83333 16.9686 6.10582C17.439 6.3455 17.8215 6.72795 18.0612 7.19836C18.3337 7.73314 18.3337 8.4332 18.3337 9.83333V13.5C18.3337 14.9001 18.3337 15.6002 18.0612 16.135C17.8215 16.6054 17.439 16.9878 16.9686 17.2275C16.4339 17.5 15.7338 17.5 14.3337 17.5H5.66699C4.26686 17.5 3.5668 17.5 3.03202 17.2275C2.56161 16.9878 2.17916 16.6054 1.93948 16.135C1.66699 15.6002 1.66699 14.9001 1.66699 13.5V5.83333Z" stroke="currentColor" stroke-width="1.66667" stroke-linecap="round" stroke-linejoin="round" />
</svg>;
