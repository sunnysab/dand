function convertDateToDisplayString(now, dateStrToConvert) {
    const target = new Date(dateStrToConvert);
    const diff = now - target;

    let description;  // 计算时间差对应的描述
    if (diff < 60 * 1000) {
        description = "刚刚";
    } else if (diff < 60 * 60 * 1000) {
        const minutes = Math.floor(diff / (60 * 1000));
        description = minutes + "分钟前";
    } else if (diff < 24 * 60 * 60 * 1000) {
        const hours = Math.floor(diff / (60 * 60 * 1000));
        description = hours + "小时前";
    } else if (diff < 48 * 60 * 60 * 1000) {
        description = "昨天";
    } else if (diff < 72 * 60 * 60 * 1000) {
        description = "前天";
    } else {
        const year = target.getFullYear();
        const month = target.getMonth() + 1;
        const day = target.getDate();

        if (year == now.getFullYear()) {
            description = month + " 月 " + day + " 日";
        } else {
            description = year + " 年 " + month + " 月 " + day + " 日";
        }
    }
    return description;
}

function replaceDateString(cssSelector) {
    // 获取当前时间
    const now = new Date();

    // 获取所有匹配的元素
    const elements = document.querySelectorAll(cssSelector);
    elements.forEach(element => {
        // 获取元素中的日期时间字符串
        const datetimeStr = element.textContent;
        const displayStr = convertDateToDisplayString(now, datetimeStr);

        element.textContent = displayStr;
    });
}

function addSectionNumber() {
    const sections = document.querySelectorAll("h2");

    const getSectionNumber = (index) => {
        const chineseNumbers = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
        if (index <= 10) {
            return chineseNumbers[index];
        } else if (index < 20) {
            return "十" + chineseNumbers[index % 10];
        } else if (index < 100) {
            return chineseNumbers[index / 10] + "十" + chineseNumbers[index % 10];
        } else {
            return "TODO: number > 100";
        }
    };

    for (var i = 0; i < sections.length; ++i) {
        const raw = sections[i].textContent;

        const convertedTitle = getSectionNumber(i + 1) + "、" + raw;
        sections[i].textContent = convertedTitle;
    }
}