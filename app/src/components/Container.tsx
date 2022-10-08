import React from "react";
import { Navbar } from "./Navbar";

export const Container = ({ children }: { children: React.ReactNode }) => (
  <>
    <Navbar />
    <div className="container mt-3">{children}</div>
  </>
);
