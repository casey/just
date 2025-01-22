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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="说明.html"><strong aria-hidden="true">1.</strong> 说明</a></li><li><ol class="section"><li class="chapter-item expanded "><div><strong aria-hidden="true">1.1.</strong> 安装</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="预备知识.html"><strong aria-hidden="true">1.1.1.</strong> 预备知识</a></li><li class="chapter-item expanded "><a href="安装包.html"><strong aria-hidden="true">1.1.2.</strong> 安装包</a></li><li class="chapter-item expanded "><a href="预制二进制文件.html"><strong aria-hidden="true">1.1.3.</strong> 预制二进制文件</a></li><li class="chapter-item expanded "><a href="github-actions.html"><strong aria-hidden="true">1.1.4.</strong> GitHub Actions</a></li><li class="chapter-item expanded "><a href="发布-rss-订阅.html"><strong aria-hidden="true">1.1.5.</strong> 发布 RSS 订阅</a></li><li class="chapter-item expanded "><a href="nodejs-安装.html"><strong aria-hidden="true">1.1.6.</strong> Node.js 安装</a></li></ol></li><li class="chapter-item expanded "><a href="向后兼容性.html"><strong aria-hidden="true">1.2.</strong> 向后兼容性</a></li><li class="chapter-item expanded "><a href="编辑器支持.html"><strong aria-hidden="true">1.3.</strong> 编辑器支持</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="vim-和-neovim.html"><strong aria-hidden="true">1.3.1.</strong> Vim 和 Neovim</a></li><li class="chapter-item expanded "><a href="emacs.html"><strong aria-hidden="true">1.3.2.</strong> Emacs</a></li><li class="chapter-item expanded "><a href="visual-studio-code.html"><strong aria-hidden="true">1.3.3.</strong> Visual Studio Code</a></li><li class="chapter-item expanded "><a href="jetbrains-ides.html"><strong aria-hidden="true">1.3.4.</strong> JetBrains IDEs</a></li><li class="chapter-item expanded "><a href="kakoune.html"><strong aria-hidden="true">1.3.5.</strong> Kakoune</a></li><li class="chapter-item expanded "><a href="sublime-text.html"><strong aria-hidden="true">1.3.6.</strong> Sublime Text</a></li><li class="chapter-item expanded "><a href="其它编辑器.html"><strong aria-hidden="true">1.3.7.</strong> 其它编辑器</a></li></ol></li><li class="chapter-item expanded "><a href="快速开始.html"><strong aria-hidden="true">1.4.</strong> 快速开始</a></li><li class="chapter-item expanded "><a href="示例.html"><strong aria-hidden="true">1.5.</strong> 示例</a></li><li class="chapter-item expanded "><div><strong aria-hidden="true">1.6.</strong> 特性介绍</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="默认配方.html"><strong aria-hidden="true">1.6.1.</strong> 默认配方</a></li><li class="chapter-item expanded "><a href="列出可用的配方.html"><strong aria-hidden="true">1.6.2.</strong> 列出可用的配方</a></li><li class="chapter-item expanded "><a href="别名.html"><strong aria-hidden="true">1.6.3.</strong> 别名</a></li><li class="chapter-item expanded "><a href="设置.html"><strong aria-hidden="true">1.6.4.</strong> 设置</a></li><li class="chapter-item expanded "><a href="文档注释.html"><strong aria-hidden="true">1.6.5.</strong> 文档注释</a></li><li class="chapter-item expanded "><a href="变量和替换.html"><strong aria-hidden="true">1.6.6.</strong> 变量和替换</a></li><li class="chapter-item expanded "><a href="字符串.html"><strong aria-hidden="true">1.6.7.</strong> 字符串</a></li><li class="chapter-item expanded "><a href="错误忽略.html"><strong aria-hidden="true">1.6.8.</strong> 错误忽略</a></li><li class="chapter-item expanded "><a href="函数.html"><strong aria-hidden="true">1.6.9.</strong> 函数</a></li><li class="chapter-item expanded "><a href="配方属性.html"><strong aria-hidden="true">1.6.10.</strong> 配方属性</a></li><li class="chapter-item expanded "><a href="使用反引号的命令求值.html"><strong aria-hidden="true">1.6.11.</strong> 使用反引号的命令求值</a></li><li class="chapter-item expanded "><a href="条件表达式.html"><strong aria-hidden="true">1.6.12.</strong> 条件表达式</a></li><li class="chapter-item expanded "><a href="出现错误停止执行.html"><strong aria-hidden="true">1.6.13.</strong> 出现错误停止执行</a></li><li class="chapter-item expanded "><a href="从命令行设置变量.html"><strong aria-hidden="true">1.6.14.</strong> 从命令行设置变量</a></li><li class="chapter-item expanded "><a href="获取和设置环境变量.html"><strong aria-hidden="true">1.6.15.</strong> 获取和设置环境变量</a></li><li class="chapter-item expanded "><a href="配方参数.html"><strong aria-hidden="true">1.6.16.</strong> 配方参数</a></li><li class="chapter-item expanded "><a href="在配方的末尾运行配方.html"><strong aria-hidden="true">1.6.17.</strong> 在配方的末尾运行配方</a></li><li class="chapter-item expanded "><a href="在配方中间运行配方.html"><strong aria-hidden="true">1.6.18.</strong> 在配方中间运行配方</a></li><li class="chapter-item expanded "><a href="用其他语言书写配方.html"><strong aria-hidden="true">1.6.19.</strong> 用其他语言书写配方</a></li><li class="chapter-item expanded "><a href="更加安全的-bash-shebang-配方.html"><strong aria-hidden="true">1.6.20.</strong> 更加安全的 Bash Shebang 配方</a></li><li class="chapter-item expanded "><a href="在配方中设置变量.html"><strong aria-hidden="true">1.6.21.</strong> 在配方中设置变量</a></li><li class="chapter-item expanded "><a href="在配方之间共享环境变量.html"><strong aria-hidden="true">1.6.22.</strong> 在配方之间共享环境变量</a></li><li class="chapter-item expanded "><a href="改变配方中的工作目录.html"><strong aria-hidden="true">1.6.23.</strong> 改变配方中的工作目录</a></li><li class="chapter-item expanded "><a href="缩进.html"><strong aria-hidden="true">1.6.24.</strong> 缩进</a></li><li class="chapter-item expanded "><a href="多行结构.html"><strong aria-hidden="true">1.6.25.</strong> 多行结构</a></li><li class="chapter-item expanded "><a href="命令行选项.html"><strong aria-hidden="true">1.6.26.</strong> 命令行选项</a></li><li class="chapter-item expanded "><a href="私有配方.html"><strong aria-hidden="true">1.6.27.</strong> 私有配方</a></li><li class="chapter-item expanded "><a href="安静配方.html"><strong aria-hidden="true">1.6.28.</strong> 安静配方</a></li><li class="chapter-item expanded "><a href="通过交互式选择器选择要运行的配方.html"><strong aria-hidden="true">1.6.29.</strong> 通过交互式选择器选择要运行的配方</a></li><li class="chapter-item expanded "><a href="在其他目录下调用-justfile.html"><strong aria-hidden="true">1.6.30.</strong> 在其他目录下调用 justfile</a></li><li class="chapter-item expanded "><a href="隐藏-justfile.html"><strong aria-hidden="true">1.6.31.</strong> 隐藏 justfile</a></li><li class="chapter-item expanded "><a href="just-脚本.html"><strong aria-hidden="true">1.6.32.</strong> Just 脚本</a></li><li class="chapter-item expanded "><a href="将-justfile-转为json文件.html"><strong aria-hidden="true">1.6.33.</strong> 将 justfile 转为JSON文件</a></li><li class="chapter-item expanded "><a href="回退到父-justfile.html"><strong aria-hidden="true">1.6.34.</strong> 回退到父 justfile</a></li><li class="chapter-item expanded "><a href="避免参数分割.html"><strong aria-hidden="true">1.6.35.</strong> 避免参数分割</a></li><li class="chapter-item expanded "><a href="配置-shell.html"><strong aria-hidden="true">1.6.36.</strong> 配置 Shell</a></li></ol></li><li class="chapter-item expanded "><a href="更新日志.html"><strong aria-hidden="true">1.7.</strong> 更新日志</a></li><li class="chapter-item expanded "><div><strong aria-hidden="true">1.8.</strong> 杂项</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="配套工具.html"><strong aria-hidden="true">1.8.1.</strong> 配套工具</a></li><li class="chapter-item expanded "><a href="并行运行任务.html"><strong aria-hidden="true">1.8.2.</strong> 并行运行任务</a></li><li class="chapter-item expanded "><a href="shell-别名.html"><strong aria-hidden="true">1.8.3.</strong> Shell 别名</a></li><li class="chapter-item expanded "><a href="shell-自动补全脚本.html"><strong aria-hidden="true">1.8.4.</strong> Shell 自动补全脚本</a></li><li class="chapter-item expanded "><a href="语法.html"><strong aria-hidden="true">1.8.5.</strong> 语法</a></li><li class="chapter-item expanded "><a href="justsh.html"><strong aria-hidden="true">1.8.6.</strong> just.sh</a></li><li class="chapter-item expanded "><a href="用户-justfile.html"><strong aria-hidden="true">1.8.7.</strong> 用户 justfile</a></li><li class="chapter-item expanded "><a href="nodejs-packagejson-脚本兼容性.html"><strong aria-hidden="true">1.8.8.</strong> Node.js package.json 脚本兼容性</a></li><li class="chapter-item expanded "><a href="替代方案.html"><strong aria-hidden="true">1.8.9.</strong> 替代方案</a></li></ol></li><li class="chapter-item expanded "><a href="贡献.html"><strong aria-hidden="true">1.9.</strong> 贡献</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="janus.html"><strong aria-hidden="true">1.9.1.</strong> Janus</a></li><li class="chapter-item expanded "><a href="最小支持的-rust-版本.html"><strong aria-hidden="true">1.9.2.</strong> 最小支持的 Rust 版本</a></li><li class="chapter-item expanded "><a href="新版本.html"><strong aria-hidden="true">1.9.3.</strong> 新版本</a></li></ol></li><li class="chapter-item expanded "><div><strong aria-hidden="true">1.10.</strong> 常见问题</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="just-避免了-make-的哪些特异性.html"><strong aria-hidden="true">1.10.1.</strong> Just 避免了 Make 的哪些特异性？</a></li><li class="chapter-item expanded "><a href="just-和-cargo-构建脚本之间有什么关系.html"><strong aria-hidden="true">1.10.2.</strong> Just 和 Cargo 构建脚本之间有什么关系？</a></li></ol></li><li class="chapter-item expanded "><a href="进一步漫谈.html"><strong aria-hidden="true">1.11.</strong> 进一步漫谈</a></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
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
