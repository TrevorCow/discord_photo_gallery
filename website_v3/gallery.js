function from_url() {
    var url_params = new URLSearchParams(location.search);
    var test = url_params.get('gallery_page_info');
    var gallery_page_from_url = JSON.parse(test);
    build_dom_from_gallery_page(gallery_page_from_url);
}
function build_dom_from_gallery_page(gallery_page) {
    // Get galleries div
    // For each gallery add h1
    // then add images
    var galleries_div = document.getElementById("galleries");
    galleries_div.innerHTML = "";
    var galleries_page_title_h1 = document.createElement("h1");
    galleries_page_title_h1.innerText = gallery_page.page_title;
    galleries_div.appendChild(galleries_page_title_h1);
    for (var _i = 0, _a = gallery_page.galleries; _i < _a.length; _i++) {
        var gallery = _a[_i];
        var gallery_title_h1 = document.createElement("h1");
        gallery_title_h1.textContent = gallery.gallery_title;
        galleries_div.appendChild(gallery_title_h1);
        var gallery_div = document.createElement("div");
        gallery_div.classList.add("gallery");
        for (var _b = 0, _c = gallery.gallery_picture_infos; _b < _c.length; _b++) {
            var gallery_image = _c[_b];
            var img = document.createElement("img");
            if (gallery_image.description) {
                img.dataset.disc = gallery_image.description;
            }
            img.dataset.discord_url = gallery_image.discord_url.toString();
            img.src = gallery_image.thumbnail_url.toString();
            gallery_div.appendChild(img);
        }
        galleries_div.appendChild(gallery_div);
    }
    var gallery_info_h3 = document.getElementById("gallery_info");
    gallery_info_h3.innerText = "Gallery was built on ".concat(gallery_page.page_built_time);
    on_dom_finished();
}
// Gallery Viewing Code
// document.addEventListener('readystatechange', () => {
//     if (document.readyState === "interactive") {
//         on_dom_finished();
//     }
// });
function on_dom_finished() {
    setup_gallery();
    setup_tool_tips();
}
function show_preview(gimp) {
    var preview_div = document.querySelector("#preview");
    if (preview_div.children.length === 0) {
        hide_tooltip_div();
        var previewImg = new Image();
        previewImg.src = gimp.dataset.discord_url;
        preview_div.appendChild(previewImg);
        preview_div.style.display = "block";
    }
    else {
        preview_div.style.display = "none";
        preview_div.innerHTML = "";
    }
}
function setup_gallery() {
    var allGalleries = document.querySelectorAll(".gallery");
    var allGalleryImages = document.querySelectorAll(".gallery img");
    var resizeAllGalleries = function () {
        allGalleries.forEach(function (gallery) {
            resizeGalleryItems(gallery);
        });
    };
    var onGalleryImageLoaded = function (gimg) {
        gimg.style.display = "inline";
        gimg.addEventListener("click", function (_event) {
            show_preview(gimg);
        });
        resizeAllGalleries();
    };
    allGalleryImages.forEach(function (gimg) {
        if (gimg.complete) {
            onGalleryImageLoaded(gimg);
        }
        else {
            gimg.addEventListener("load", function (_event) {
                onGalleryImageLoaded(gimg);
            });
            gimg.addEventListener('error', function (err) {
                console.log(err);
            });
        }
    });
    window.addEventListener("resize", function (_event) {
        resizeAllGalleries();
    });
}
function resizeGalleryItems(gallery) {
    var children = gallery.querySelectorAll("img");
    children.forEach(function (child) {
        console.assert(gallery.classList.contains("gallery"));
        var computedGalleryStyle = window.getComputedStyle(gallery);
        var rowHeight = parseInt(computedGalleryStyle.getPropertyValue('grid-auto-rows'));
        var rowGap = parseInt(computedGalleryStyle.getPropertyValue('grid-row-gap'));
        var rowSpan = Math.ceil((child.getBoundingClientRect().height + rowGap) / (rowHeight + rowGap));
        child.style.gridRowEnd = "span " + rowSpan;
    });
}
function hide_tooltip_div() {
    var tooltip = document.querySelector("#tooltip");
    tooltip.style.display = "none";
}
function setup_tool_tips() {
    var tooltip = document.querySelector("#tooltip");
    function onmm(e) {
        var parentContent = e.currentTarget;
        var newX = e.clientX + 10;
        var newY = e.clientY + 10;
        tooltip.style.top = newY + 'px';
        tooltip.style.left = newX + 'px';
        tooltip.style.display = "block";
        tooltip.innerText = parentContent.dataset.disc;
    }
    var images_that_need_tooltip = document.querySelectorAll(".gallery img");
    images_that_need_tooltip.forEach(function (content_object) {
        if (content_object.dataset.disc) {
            content_object.addEventListener("mousemove", onmm, false);
        }
        content_object.addEventListener("mouseleave", function (_e) {
            hide_tooltip_div();
        }, false);
    });
}
