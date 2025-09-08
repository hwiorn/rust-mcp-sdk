// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="title-page.html">The PMCP Guide</a></li><li class="chapter-item expanded affix "><a href="foreword.html">Foreword</a></li><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded "><a href="ch01-installation.html"><strong aria-hidden="true">1.</strong> Chapter 1: Installation &amp; Setup</a></li><li class="chapter-item expanded "><a href="ch02-first-server.html"><strong aria-hidden="true">2.</strong> Chapter 2: Your First MCP Server</a></li><li class="chapter-item expanded "><a href="ch03-first-client.html"><strong aria-hidden="true">3.</strong> Chapter 3: Your First MCP Client</a></li><li class="chapter-item expanded "><a href="ch04-protocol-basics.html"><strong aria-hidden="true">4.</strong> Chapter 4: Understanding the Protocol</a></li><li class="chapter-item expanded "><a href="ch05-tools.html"><strong aria-hidden="true">5.</strong> Chapter 5: Tools &amp; Tool Handlers</a></li><li class="chapter-item expanded "><a href="ch06-resources.html"><strong aria-hidden="true">6.</strong> Chapter 6: Resources &amp; Resource Management</a></li><li class="chapter-item expanded "><a href="ch07-prompts.html"><strong aria-hidden="true">7.</strong> Chapter 7: Prompts &amp; Templates</a></li><li class="chapter-item expanded "><a href="ch08-error-handling.html"><strong aria-hidden="true">8.</strong> Chapter 8: Error Handling &amp; Recovery</a></li><li class="chapter-item expanded "><a href="ch09-auth-security.html"><strong aria-hidden="true">9.</strong> Chapter 9: Authentication &amp; Security</a></li><li class="chapter-item expanded "><a href="ch10-transports.html"><strong aria-hidden="true">10.</strong> Chapter 10: Transport Layers</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item expanded "><a href="ch10-01-websocket.html"><strong aria-hidden="true">10.1.</strong> WebSocket Transport</a></li><li class="chapter-item expanded "><a href="ch10-02-http.html"><strong aria-hidden="true">10.2.</strong> HTTP Transport</a></li><li class="chapter-item expanded "><a href="ch10-03-streamable-http.html"><strong aria-hidden="true">10.3.</strong> Streamable HTTP</a></li></ol></li><li class="chapter-item expanded "><a href="ch11-middleware.html"><strong aria-hidden="true">11.</strong> Chapter 11: Middleware &amp; Composition</a></li><li class="chapter-item expanded "><a href="ch12-progress-cancel.html"><strong aria-hidden="true">12.</strong> Chapter 12: Progress Tracking &amp; Cancellation</a></li><li class="chapter-item expanded "><a href="ch13-production.html"><strong aria-hidden="true">13.</strong> Chapter 13: Building Production Servers</a></li><li class="chapter-item expanded "><a href="ch14-performance.html"><strong aria-hidden="true">14.</strong> Chapter 14: Performance &amp; Optimization</a></li><li class="chapter-item expanded "><a href="ch15-testing.html"><strong aria-hidden="true">15.</strong> Chapter 15: Testing &amp; Quality Assurance</a></li><li class="chapter-item expanded "><a href="ch16-deployment.html"><strong aria-hidden="true">16.</strong> Chapter 16: Deployment Strategies</a></li><li class="chapter-item expanded "><a href="ch17-examples.html"><strong aria-hidden="true">17.</strong> Chapter 17: Complete Examples</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item expanded "><a href="ch17-01-parallel-clients.html"><strong aria-hidden="true">17.1.</strong> Multiple Parallel Clients</a></li><li class="chapter-item expanded "><a href="ch17-02-structured-output.html"><strong aria-hidden="true">17.2.</strong> Structured Output Schemas</a></li><li class="chapter-item expanded "><a href="ch17-03-sampling-tools.html"><strong aria-hidden="true">17.3.</strong> Tool with Sampling</a></li></ol></li><li class="chapter-item expanded "><a href="ch18-patterns.html"><strong aria-hidden="true">18.</strong> Chapter 18: Design Patterns</a></li><li class="chapter-item expanded "><a href="ch19-integration.html"><strong aria-hidden="true">19.</strong> Chapter 19: Integration Patterns</a></li><li class="chapter-item expanded "><a href="ch20-typescript-interop.html"><strong aria-hidden="true">20.</strong> Chapter 20: TypeScript Interoperability</a></li><li class="chapter-item expanded "><a href="ch21-migration.html"><strong aria-hidden="true">21.</strong> Chapter 21: Migration Guide</a></li><li class="chapter-item expanded "><a href="ch22-feature-parity.html"><strong aria-hidden="true">22.</strong> Chapter 22: Feature Parity</a></li><li class="chapter-item expanded "><a href="ch23-custom-transports.html"><strong aria-hidden="true">23.</strong> Chapter 23: Custom Transports</a></li><li class="chapter-item expanded "><a href="ch24-extensions.html"><strong aria-hidden="true">24.</strong> Chapter 24: Protocol Extensions</a></li><li class="chapter-item expanded "><a href="ch25-analysis.html"><strong aria-hidden="true">25.</strong> Chapter 25: Performance Analysis</a></li><li class="chapter-item expanded "><a href="ch26-contributing.html"><strong aria-hidden="true">26.</strong> Chapter 26: Contributing to PMCP</a></li><li class="chapter-item expanded "><a href="appendix-a-installation.html"><strong aria-hidden="true">27.</strong> Appendix A: Installation Guide</a></li><li class="chapter-item expanded "><a href="appendix-b-config.html"><strong aria-hidden="true">28.</strong> Appendix B: Configuration Reference</a></li><li class="chapter-item expanded "><a href="appendix-c-api.html"><strong aria-hidden="true">29.</strong> Appendix C: API Reference</a></li><li class="chapter-item expanded "><a href="appendix-d-errors.html"><strong aria-hidden="true">30.</strong> Appendix D: Error Codes</a></li><li class="chapter-item expanded "><a href="appendix-e-troubleshooting.html"><strong aria-hidden="true">31.</strong> Appendix E: Troubleshooting</a></li><li class="chapter-item expanded "><a href="appendix-f-glossary.html"><strong aria-hidden="true">32.</strong> Appendix F: Glossary</a></li><li class="chapter-item expanded "><a href="appendix-g-resources.html"><strong aria-hidden="true">33.</strong> Appendix G: Resources &amp; Links</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
