import { Routes, Route, Navigate } from "react-router-dom";
import { MainLayout } from "@/layouts/MainLayout";
import { LandingPage } from "@/pages/LandingPage";
import { DocsLayout } from "@/pages/DocsLayout";
import { DocPage } from "@/pages/DocPage";

export default function App() {
  return (
    <MainLayout>
      <Routes>
        <Route path="/" element={<LandingPage />} />
        <Route path="docs" element={<DocsLayout />}>
          <Route index element={<Navigate to="introduction" replace />} />
          <Route path=":slug" element={<DocPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </MainLayout>
  );
}
