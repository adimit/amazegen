import PDF from "pdfkit";

declare global {
  class PDFDocument extends PDF {}
}

export const withPdf = async (
  fileName: string,
  action: (pdf: PDFDocument) => void
) => {
  /*
    Using pdfkit with Vite is a pain in the ass. pdfkit uses
    lots of Node libraries, which wouldn't be a problem on its own,
    but it also uses brfs (and something else with webpack) to bake
    in some fonts. I'd have to emulate what they're doing somehow
    in Vite with both esbuild and rollup, and I'm not keen on that.

    So instead, there's this hack. We're using a dynamic import to
    load the statically generated pdfkit file. Why are we doing that?
    Because the static file clocks in at over 1.5MB, and I don't want
    to load that on page load. BUT, it seems that rollup and esbuild
    behave differently when it comes to side effects of dynamic
    imports.

    See, esbulid just executes pdfkit.js and PDFDocument is suddenly
    available in the global scope. It's as if you had <script>-ed it.
    And you *don't* get a default export from it.

    And rollup does the exact opposite. Nothing is in the global
    scope after the import finishes, and it *does* export the main
    PDFDocument class as a default export. And I have no idea which
    magic set of options would make both behave the same.

    So we do both, and just take what's there, and tell TypeScript
    to look the other way.

    This little function neatly abstracts all the horrors, so it
    will hopefully never become a problem.
  */

  // @ts-ignore dynamically load pdfkit and maybe get a default export…
  const { default: PDFKitExport } = await import("../pdfkit.js");
  // PDFKitExport is available with rollup, global PDFDocument with esbuild
  const PDF = PDFKitExport ?? PDFDocument;
  const { default: blobStream } = await import("blob-stream");
  const { saveAs } = await import("file-saver");

  const pdf = new PDF();
  const stream = pdf.pipe(blobStream());
  action(pdf);
  pdf.end();
  stream.on("finish", () => {
    const blob = stream.toBlob("application/pdf");
    saveAs(blob, `${fileName}.pdf`);
  });
};
